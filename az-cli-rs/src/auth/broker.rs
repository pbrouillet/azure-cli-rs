//! Microsoft Identity Broker client — silent token acquisition without ever
//! handling the user's credentials.
//!
//! Two platform transports sit behind one API:
//!
//! * **Linux** — the session-D-Bus `com.microsoft.identity.broker1` interface
//!   (Intune broker / himmelblau), spoken as JSON-over-D-Bus.
//! * **Windows** — the Web Account Manager (WAM) WinRT API
//!   (`Windows.Security.Authentication.Web.Core`), talked to through the
//!   `windows` crate's safe projections.
//!
//! macOS (and anything else) reports the broker as unavailable, so callers
//! fall back to the interactive / device-code flows.
//!
//! The request/response *shaping* and account-selection logic are pure and
//! unit-tested; only the transports are platform-specific.
//!
//! Ported from Paul Wiens' `pwrtools-rs` `pwr-entra` crate (MIT). The two WAM
//! fixes captured below (`FindAllAccountsWithClientIdAsync` and the ADAL
//! `resource` property) were discovered live against a real Azure-AD-joined
//! machine and are load-bearing.

// The pure helpers are consumed by the Linux transport and the unit tests; on
// other platforms they are intentionally unused.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

use crate::auth::oauth2::AZURE_CLI_CLIENT_ID;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub(crate) const BUS_NAME: &str = "com.microsoft.identity.broker1";
pub(crate) const OBJECT_PATH: &str = "/com/microsoft/identity/broker1";
pub(crate) const IFACE: &str = "com.microsoft.identity.Broker1";
pub(crate) const PROTO_VER: &str = "1.0";

/// Errors from the broker acquisition path. Converted to [`crate::error::AzrsError`]
/// at the call boundary.
#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    /// The broker is not reachable on this machine.
    #[error("Microsoft Identity Broker / WAM is not available on this machine")]
    Unavailable,
    /// A transport / protocol failure.
    #[error("broker: {0}")]
    Broker(String),
    /// The broker returned a structured error (status/substatus/context).
    #[error("broker error (status={status}, sub={sub_status}): {context}")]
    BrokerStatus {
        /// Broker status code.
        status: i64,
        /// Broker sub-status code.
        sub_status: i64,
        /// Human-readable context.
        context: String,
    },
    /// No accounts are registered with the broker.
    #[error("no accounts registered with the broker")]
    NoAccounts,
    /// The requested account was not found among broker accounts.
    #[error("account {username:?} not found in broker (available: {available})")]
    AccountNotFound {
        /// The account that was requested.
        username: String,
        /// Comma-separated list of available usernames.
        available: String,
    },
    /// A response could not be parsed.
    #[error("parse: {0}")]
    Parse(String),
}

impl From<BrokerError> for crate::error::AzrsError {
    fn from(e: BrokerError) -> Self {
        crate::error::AzrsError::Auth(e.to_string())
    }
}

/// A successfully acquired access token and its expiry.
#[derive(Debug, Clone)]
pub struct BrokerToken {
    /// The bearer access token (a JWT).
    pub access_token: String,
    /// Absolute expiry time.
    pub expires_on: DateTime<Utc>,
}

impl BrokerToken {
    /// Build from a raw token, preferring the JWT `exp` claim for the expiry
    /// and falling back to `fallback` (the broker's claimed expiry). A numeric
    /// fallback less than 60s out is treated as bogus (himmelblau has a
    /// seconds-vs-millis bug) and pinned to one hour from now.
    fn from_token(access_token: String, fallback: Option<DateTime<Utc>>) -> Self {
        let expires_on = match token_expiry(&access_token) {
            Some(exp) => exp,
            None => match fallback {
                Some(f) if f > Utc::now() + chrono::Duration::seconds(60) => f,
                _ => Utc::now() + chrono::Duration::seconds(3600),
            },
        };
        BrokerToken {
            access_token,
            expires_on,
        }
    }
}

/// Configuration for a broker token request.
#[derive(Debug, Clone)]
pub struct BrokerConfig {
    /// App registration client id.
    pub client_id: String,
    /// Redirect URI registered on the app.
    pub redirect_uri: String,
    /// Authority URL, e.g. `https://login.microsoftonline.com/<tenant>`.
    pub authority: String,
    /// Requested OAuth scopes.
    pub scopes: Vec<String>,
    /// Account selector (`user@tenant.com`). Empty selects the sole/first account.
    pub username: String,
    /// Optional tenant GUID to disambiguate a multi-realm account.
    pub tenant: Option<String>,
}

impl BrokerConfig {
    /// Build an ARM-scoped broker config for the Azure CLI public client.
    pub fn for_arm(authority: String, scopes: Vec<String>, username: String, tenant: Option<String>) -> Self {
        BrokerConfig {
            client_id: AZURE_CLI_CLIENT_ID.to_string(),
            redirect_uri: format!("ms-appx-web://Microsoft.AAD.BrokerPlugin/{AZURE_CLI_CLIENT_ID}"),
            authority,
            scopes,
            username,
            tenant,
        }
    }
}

/// A broker account record (platform-neutral).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// `{oid}.{home-tenant-guid}`.
    #[serde(rename = "homeAccountId", default)]
    pub home_account_id: String,
    /// e.g. `login.microsoftonline.com`.
    #[serde(default)]
    pub environment: String,
    /// The tenant GUID this record is enrolled into (blank on WAM).
    #[serde(default)]
    pub realm: String,
    /// Object id within the realm.
    #[serde(rename = "localAccountId", default)]
    pub local_account_id: String,
    /// `user@tenant.com`.
    #[serde(default)]
    pub username: String,
}

#[derive(Serialize)]
struct GetAccountsRequest<'a> {
    #[serde(rename = "clientId")]
    client_id: &'a str,
    #[serde(rename = "redirectUri")]
    redirect_uri: &'a str,
}

#[derive(Deserialize)]
struct GetAccountsResponse {
    #[serde(default)]
    accounts: Vec<Account>,
}

#[derive(Serialize)]
struct AuthParameters<'a> {
    account: &'a Account,
    authority: &'a str,
    #[serde(rename = "clientId")]
    client_id: &'a str,
    #[serde(rename = "redirectUri")]
    redirect_uri: &'a str,
    #[serde(rename = "requestedScopes")]
    requested_scopes: &'a [String],
    #[serde(rename = "authorizationType")]
    authorization_type: i32,
    username: &'a str,
    #[serde(rename = "uxContextHandle")]
    ux_context_handle: i32,
}

#[derive(Serialize)]
struct AcquireTokenRequest<'a> {
    account: &'a Account,
    #[serde(rename = "authParameters")]
    auth_parameters: AuthParameters<'a>,
}

#[derive(Deserialize)]
struct BrokerErrorResponse {
    #[serde(default)]
    context: String,
    #[serde(default)]
    status: i64,
    #[serde(rename = "subStatus", default)]
    sub_status: i64,
}

#[derive(Deserialize)]
struct BrokerTokenResponse {
    #[serde(rename = "brokerTokenResponse")]
    inner: BrokerTokenInner,
}

#[derive(Deserialize)]
struct BrokerTokenInner {
    #[serde(rename = "accessToken", default)]
    access_token: String,
    #[serde(rename = "expiresOn", default)]
    expires_on: i64,
    #[serde(default)]
    error: Option<BrokerErrorResponse>,
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-tested)
// ---------------------------------------------------------------------------

/// Minimal JWT `exp` extraction — never verifies the signature (the token came
/// from the broker or AAD and is trusted). Returns the `exp` as an absolute
/// time, or `None` if the token isn't a parseable JWT or carries no `exp`.
fn token_expiry(raw: &str) -> Option<DateTime<Utc>> {
    let claims = decode_claims(raw)?;
    match claims.get("exp") {
        Some(serde_json::Value::Number(n)) => {
            let exp = n.as_i64().filter(|&e| e > 0)?;
            DateTime::from_timestamp(exp, 0)
        }
        _ => None,
    }
}

/// Return the token's `tid` (directory/tenant GUID) claim, or `None`.
pub fn token_tenant(raw: &str) -> Option<String> {
    let claims = decode_claims(raw)?;
    match claims.get("tid") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

fn decode_claims(raw: &str) -> Option<serde_json::Value> {
    use base64::Engine;
    let seg = raw.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(seg)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(seg))
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(seg))
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Parse a `getAccounts` response body into accounts.
fn parse_accounts(json: &str) -> Result<Vec<Account>, BrokerError> {
    serde_json::from_str::<GetAccountsResponse>(json)
        .map(|r| r.accounts)
        .map_err(|e| BrokerError::Parse(format!("getAccounts response: {e}")))
}

/// Choose the account matching `cfg.username`, preferring the home-tenant realm
/// (or `cfg.tenant` when set) if the username spans multiple realms.
fn select_account(accounts: Vec<Account>, cfg: &BrokerConfig) -> Result<Account, BrokerError> {
    if accounts.is_empty() {
        return Err(BrokerError::NoAccounts);
    }
    if cfg.username.is_empty() {
        return Ok(accounts.into_iter().next().unwrap());
    }
    let matches: Vec<Account> = accounts
        .iter()
        .filter(|a| a.username.eq_ignore_ascii_case(&cfg.username))
        .cloned()
        .collect();
    if matches.is_empty() {
        let available = accounts
            .iter()
            .map(|a| a.username.clone())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(BrokerError::AccountNotFound {
            username: cfg.username.clone(),
            available,
        });
    }
    Ok(pick_preferred_realm(matches, cfg.tenant.as_deref()))
}

/// Among accounts sharing a username, prefer: the explicit tenant, then the
/// home tenant encoded in `homeAccountId` (`{oid}.{home-tid}`), then the first.
fn pick_preferred_realm(mut matches: Vec<Account>, prefer_tenant: Option<&str>) -> Account {
    if matches.len() == 1 {
        return matches.remove(0);
    }
    if let Some(tenant) = prefer_tenant.filter(|t| !t.is_empty()) {
        if let Some(pos) = matches
            .iter()
            .position(|m| m.realm.eq_ignore_ascii_case(tenant))
        {
            return matches.remove(pos);
        }
    }
    if let Some(pos) = matches.iter().position(|m| {
        m.home_account_id
            .split_once('.')
            .is_some_and(|(_, home_tid)| m.realm.eq_ignore_ascii_case(home_tid))
    }) {
        return matches.remove(pos);
    }
    matches.remove(0)
}

/// Turn a broker token response into a [`BrokerToken`], preferring the JWT's
/// `exp` claim over the broker's (sometimes buggy) `expiresOn` (in millis).
fn parse_token_response(json: &str) -> Result<BrokerToken, BrokerError> {
    let resp: BrokerTokenResponse = serde_json::from_str(json)
        .map_err(|e| BrokerError::Parse(format!("token response: {e}")))?;
    if let Some(err) = resp.inner.error {
        return Err(BrokerError::BrokerStatus {
            status: err.status,
            sub_status: err.sub_status,
            context: err.context,
        });
    }
    if resp.inner.access_token.is_empty() {
        return Err(BrokerError::Broker(
            "empty access token in broker response".into(),
        ));
    }
    let fallback =
        (resp.inner.expires_on > 0).then(|| DateTime::from_timestamp_millis(resp.inner.expires_on)).flatten();
    Ok(BrokerToken::from_token(resp.inner.access_token, fallback))
}

fn auth_parameters<'a>(
    acct: &'a Account,
    cfg: &'a BrokerConfig,
    authorization_type: i32,
) -> AuthParameters<'a> {
    AuthParameters {
        account: acct,
        authority: &cfg.authority,
        client_id: &cfg.client_id,
        redirect_uri: &cfg.redirect_uri,
        requested_scopes: &cfg.scopes,
        authorization_type,
        username: &acct.username,
        ux_context_handle: -1,
    }
}

// ---------------------------------------------------------------------------
// Linux D-Bus transport
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod transport {
    use super::*;
    use std::time::Duration;
    use zbus::blocking::Connection;

    const TRANSIENT_RETRY: Duration = Duration::from_secs(2);

    fn is_connect_error(err: &zbus::Error) -> bool {
        err.to_string().contains("broker socket connect")
    }

    fn is_timeout_error(err: &zbus::Error) -> bool {
        err.to_string()
            .contains("Socket timeout waiting for daemon response")
    }

    pub(super) fn connect() -> Result<Connection, BrokerError> {
        Connection::session().map_err(|e| BrokerError::Broker(format!("connect session bus: {e}")))
    }

    /// Invoke a broker method. Retries once on a transient IPC error. When
    /// `allow_timeout_retry` is false (interactive methods), a response timeout
    /// is *not* retried — it could double-pop the auth dialog.
    pub(super) fn call(
        conn: &Connection,
        method: &str,
        request_json: &str,
        allow_timeout_retry: bool,
    ) -> Result<String, BrokerError> {
        let corr = uuid::Uuid::new_v4().to_string();
        let invoke = || {
            conn.call_method(
                Some(BUS_NAME),
                OBJECT_PATH,
                Some(IFACE),
                method,
                &(PROTO_VER, corr.as_str(), request_json),
            )
        };
        let should_retry =
            |e: &zbus::Error| is_connect_error(e) || (allow_timeout_retry && is_timeout_error(e));
        let reply = match invoke() {
            Ok(r) => r,
            Err(e) if should_retry(&e) => {
                std::thread::sleep(TRANSIENT_RETRY);
                invoke().map_err(|e| BrokerError::Broker(format!("D-Bus call {method}: {e}")))?
            }
            Err(e) => return Err(BrokerError::Broker(format!("D-Bus call {method}: {e}"))),
        };
        reply
            .body()
            .deserialize::<String>()
            .map_err(|e| BrokerError::Parse(format!("D-Bus reply: {e}")))
    }

    pub(super) fn is_broker_available() -> bool {
        let Ok(conn) = Connection::session() else {
            return false;
        };
        let Ok(proxy) = zbus::blocking::fdo::DBusProxy::new(&conn) else {
            return false;
        };
        if let Ok(names) = proxy.list_activatable_names() {
            if names.iter().any(|n| n.as_str() == BUS_NAME) {
                return true;
            }
        }
        matches!(proxy.name_has_owner(BUS_NAME.try_into().unwrap()), Ok(true))
    }

    pub(super) fn fetch_accounts(
        conn: &Connection,
        client_id: &str,
        redirect_uri: &str,
    ) -> Result<Vec<Account>, BrokerError> {
        let req = serde_json::to_string(&GetAccountsRequest {
            client_id,
            redirect_uri,
        })
        .map_err(|e| BrokerError::Parse(e.to_string()))?;
        let resp = call(conn, "getAccounts", &req, true)?;
        parse_accounts(&resp)
    }
}

// ---------------------------------------------------------------------------
// Windows Web Account Manager (WAM) transport
// ---------------------------------------------------------------------------
//
// Two behaviors here were only discovered by testing live against a real
// Azure-AD-joined machine (they aren't documented clearly anywhere):
//
// * `FindAllAccountsAsync` (no client id) reliably comes back
//   `ProviderError` with zero accounts for the AAD provider; enumerating
//   accounts requires the client-id-scoped `FindAllAccountsWithClientIdAsync`.
// * A `WebTokenRequest`'s `scope` field alone is not enough for AAD — WAM also
//   needs an ADAL-style `resource` property (the scope with its trailing
//   `/.default` or path stripped) or it silently resolves the wrong (legacy)
//   resource and every call fails with AADSTS65002.
#[cfg(target_os = "windows")]
mod wintransport {
    use super::*;
    use windows::core::{Error as WinError, HSTRING};
    use windows::Security::Authentication::Web::Core::{
        WebAuthenticationCoreManager, WebTokenRequest, WebTokenRequestStatus,
    };
    use windows::Security::Credentials::{WebAccount, WebAccountProvider};

    /// The WAM provider id for Azure AD / Microsoft Entra accounts.
    const AAD_PROVIDER_ID: &str = "https://login.microsoft.com";

    fn win_err(context: &str, e: WinError) -> BrokerError {
        BrokerError::Broker(format!("{context}: {e} (0x{:08X})", e.code().0 as u32))
    }

    /// The full authority URL WAM expects, built from `cfg.tenant` when set,
    /// else `cfg.authority` as-is, else the multi-tenant default.
    pub(super) fn provider_authority(cfg: &BrokerConfig) -> String {
        if let Some(t) = cfg.tenant.as_deref().filter(|t| !t.is_empty()) {
            return format!("https://login.microsoftonline.com/{t}");
        }
        if cfg.authority.is_empty() {
            return "https://login.microsoftonline.com/organizations".to_string();
        }
        cfg.authority.clone()
    }

    /// Derive the ADAL-style `resource` from the first requested scope,
    /// stripping its trailing `/.default` or path segment.
    fn resource_from_scopes(scopes: &[String]) -> Option<String> {
        let first = scopes.first()?;
        match first.rsplit_once('/') {
            Some((base, _)) if !base.is_empty() => Some(base.to_string()),
            _ => Some(first.clone()),
        }
    }

    fn find_provider(authority: &str) -> Result<WebAccountProvider, BrokerError> {
        WebAuthenticationCoreManager::FindAccountProviderWithAuthorityAsync(
            &HSTRING::from(AAD_PROVIDER_ID),
            &HSTRING::from(authority),
        )
        .and_then(|op| op.get())
        .map_err(|e| win_err("find account provider", e))
    }

    /// Enumerate WAM accounts for `provider`, scoped to a client id (the
    /// unscoped `FindAllAccountsAsync` returns `ProviderError` on real hardware).
    fn all_accounts(
        provider: &WebAccountProvider,
        client_id: &str,
    ) -> Result<Vec<WebAccount>, BrokerError> {
        let result = WebAuthenticationCoreManager::FindAllAccountsWithClientIdAsync(
            provider,
            &HSTRING::from(client_id),
        )
        .and_then(|op| op.get())
        .map_err(|e| win_err("find all accounts", e))?;
        let view = result.Accounts().map_err(|e| win_err("read accounts", e))?;
        let size = view.Size().map_err(|e| win_err("account count", e))?;
        let mut out = Vec::with_capacity(size as usize);
        for i in 0..size {
            out.push(view.GetAt(i).map_err(|e| win_err("account at index", e))?);
        }
        Ok(out)
    }

    /// Map a native `WebAccount` into the platform-neutral [`Account`] shape.
    /// WAM exposes no realm/tenant field on the account, so `realm` is blank.
    fn to_account(a: &WebAccount) -> Account {
        let username = a.UserName().map(|h| h.to_string_lossy()).unwrap_or_default();
        let id = a.Id().map(|h| h.to_string_lossy()).unwrap_or_default();
        Account {
            home_account_id: id.clone(),
            environment: "login.microsoftonline.com".to_string(),
            realm: String::new(),
            local_account_id: id,
            username,
        }
    }

    fn select(accounts: Vec<WebAccount>, username: &str) -> Result<WebAccount, BrokerError> {
        if accounts.is_empty() {
            return Err(BrokerError::NoAccounts);
        }
        if username.is_empty() {
            return Ok(accounts.into_iter().next().unwrap());
        }
        let available: Vec<String> = accounts.iter().map(|a| to_account(a).username).collect();
        accounts
            .into_iter()
            .find(|a| {
                a.UserName()
                    .map(|u| u.to_string_lossy().eq_ignore_ascii_case(username))
                    .unwrap_or(false)
            })
            .ok_or_else(|| BrokerError::AccountNotFound {
                username: username.to_string(),
                available: available.join(", "),
            })
    }

    pub(super) fn is_available() -> bool {
        find_provider("https://login.microsoftonline.com/organizations").is_ok()
    }

    pub(super) fn list_accounts(client_id: &str) -> Result<Vec<Account>, BrokerError> {
        let provider = find_provider("https://login.microsoftonline.com/organizations")?;
        Ok(all_accounts(&provider, client_id)?.iter().map(to_account).collect())
    }

    fn build_request(
        provider: &WebAccountProvider,
        cfg: &BrokerConfig,
    ) -> Result<WebTokenRequest, BrokerError> {
        let scope = cfg.scopes.join(" ");
        let request = WebTokenRequest::Create(
            provider,
            &HSTRING::from(scope.as_str()),
            &HSTRING::from(cfg.client_id.as_str()),
        )
        .map_err(|e| win_err("build token request", e))?;
        if let Some(resource) = resource_from_scopes(&cfg.scopes) {
            let props = request
                .Properties()
                .map_err(|e| win_err("token request properties", e))?;
            props
                .Insert(&HSTRING::from("resource"), &HSTRING::from(resource.as_str()))
                .map_err(|e| win_err("set resource property", e))?;
        }
        Ok(request)
    }

    pub(super) fn acquire_token(
        cfg: &BrokerConfig,
        interactive: bool,
    ) -> Result<BrokerToken, BrokerError> {
        let provider = find_provider(&provider_authority(cfg))?;
        let request = build_request(&provider, cfg)?;

        let result = if interactive {
            WebAuthenticationCoreManager::RequestTokenAsync(&request)
                .and_then(|op| op.get())
                .map_err(|e| win_err("request token", e))?
        } else {
            let accounts = all_accounts(&provider, &cfg.client_id)?;
            let account = select(accounts, &cfg.username)?;
            WebAuthenticationCoreManager::GetTokenSilentlyWithWebAccountAsync(&request, &account)
                .and_then(|op| op.get())
                .map_err(|e| win_err("get token silently", e))?
        };

        match result.ResponseStatus().map_err(|e| win_err("response status", e))? {
            WebTokenRequestStatus::Success => {
                let data = result.ResponseData().map_err(|e| win_err("response data", e))?;
                let first = data.GetAt(0).map_err(|e| win_err("first response", e))?;
                let token = first.Token().map_err(|e| win_err("token", e))?.to_string_lossy();
                let fallback = first
                    .Properties()
                    .ok()
                    .and_then(|p| p.Lookup(&HSTRING::from("ExpiresOn")).ok())
                    .and_then(|s| s.to_string_lossy().parse::<i64>().ok())
                    .and_then(|secs| DateTime::from_timestamp(secs, 0));
                Ok(BrokerToken::from_token(token, fallback))
            }
            status => {
                let (sub_status, context) = result
                    .ResponseError()
                    .ok()
                    .map(|e| {
                        (
                            e.ErrorCode().unwrap_or_default() as i64,
                            e.ErrorMessage().map(|m| m.to_string_lossy()).unwrap_or_default(),
                        )
                    })
                    .unwrap_or_default();
                Err(BrokerError::BrokerStatus {
                    status: status.0 as i64,
                    sub_status,
                    context,
                })
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn cfg(authority: &str, tenant: Option<&str>) -> BrokerConfig {
            BrokerConfig {
                client_id: "cid".into(),
                redirect_uri: "http://localhost".into(),
                authority: authority.into(),
                scopes: vec![],
                username: String::new(),
                tenant: tenant.map(str::to_string),
            }
        }

        #[test]
        fn provider_authority_prefers_explicit_tenant() {
            let c = cfg("https://login.microsoftonline.com/organizations", Some("tid"));
            assert_eq!(provider_authority(&c), "https://login.microsoftonline.com/tid");
        }

        #[test]
        fn provider_authority_uses_authority_as_is() {
            let c = cfg("https://login.microsoftonline.com/common", None);
            assert_eq!(provider_authority(&c), "https://login.microsoftonline.com/common");
        }

        #[test]
        fn resource_from_scopes_strips_default_suffix() {
            assert_eq!(
                resource_from_scopes(&["https://graph.microsoft.com/.default".to_string()]),
                Some("https://graph.microsoft.com".to_string())
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Whether the Microsoft Identity Broker / WAM is reachable on this machine.
pub fn available() -> bool {
    #[cfg(target_os = "linux")]
    {
        transport::is_broker_available()
    }
    #[cfg(target_os = "windows")]
    {
        wintransport::is_available()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        false
    }
}

/// Enumerate all accounts registered with the broker.
pub fn list_accounts(redirect_uri: &str) -> Result<Vec<Account>, BrokerError> {
    #[cfg(target_os = "linux")]
    {
        let conn = transport::connect()?;
        transport::fetch_accounts(&conn, AZURE_CLI_CLIENT_ID, redirect_uri)
    }
    #[cfg(target_os = "windows")]
    {
        let _ = redirect_uri;
        wintransport::list_accounts(AZURE_CLI_CLIENT_ID)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = redirect_uri;
        Err(BrokerError::Unavailable)
    }
}

/// Acquire a token silently (no prompt) via the broker's device PRT / WAM.
pub fn acquire_token_silent(cfg: &BrokerConfig) -> Result<BrokerToken, BrokerError> {
    acquire_token(cfg, 1)
}

/// Acquire a token interactively (may pop an AAD auth UI).
pub fn acquire_token_interactive(cfg: &BrokerConfig) -> Result<BrokerToken, BrokerError> {
    acquire_token(cfg, 2)
}

#[cfg(target_os = "linux")]
fn acquire_token(cfg: &BrokerConfig, authorization_type: i32) -> Result<BrokerToken, BrokerError> {
    let conn = transport::connect()?;
    let accounts = transport::fetch_accounts(&conn, &cfg.client_id, &cfg.redirect_uri)?;
    let acct = select_account(accounts, cfg)?;
    let req = AcquireTokenRequest {
        account: &acct,
        auth_parameters: auth_parameters(&acct, cfg, authorization_type),
    };
    let req_json = serde_json::to_string(&req).map_err(|e| BrokerError::Parse(e.to_string()))?;
    let method = if authorization_type == 1 {
        "acquireTokenSilently"
    } else {
        "acquireTokenInteractively"
    };
    let resp = transport::call(&conn, method, &req_json, authorization_type == 1)?;
    parse_token_response(&resp)
}

#[cfg(target_os = "windows")]
fn acquire_token(cfg: &BrokerConfig, authorization_type: i32) -> Result<BrokerToken, BrokerError> {
    wintransport::acquire_token(cfg, authorization_type != 1)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn acquire_token(cfg: &BrokerConfig, _authorization_type: i32) -> Result<BrokerToken, BrokerError> {
    let _ = cfg;
    Err(BrokerError::Unavailable)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acct(username: &str, realm: &str, home: &str) -> Account {
        Account {
            home_account_id: home.into(),
            environment: "login.microsoftonline.com".into(),
            realm: realm.into(),
            local_account_id: "oid".into(),
            username: username.into(),
        }
    }

    fn cfg(username: &str, tenant: Option<&str>) -> BrokerConfig {
        BrokerConfig {
            client_id: "cid".into(),
            redirect_uri: "http://localhost".into(),
            authority: "https://login.microsoftonline.com/common".into(),
            scopes: vec!["s/.default".into()],
            username: username.into(),
            tenant: tenant.map(String::from),
        }
    }

    fn jwt_with(claims: &str) -> String {
        use base64::Engine;
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"alg":"none"}"#);
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.as_bytes());
        format!("{header}.{payload}.sig")
    }

    #[test]
    fn parse_accounts_ok() {
        let json = r#"{"accounts":[{"homeAccountId":"oid.tid","environment":"e","realm":"tid","localAccountId":"oid","username":"a@b.com"}]}"#;
        let accts = parse_accounts(json).unwrap();
        assert_eq!(accts.len(), 1);
        assert_eq!(accts[0].username, "a@b.com");
        assert_eq!(accts[0].realm, "tid");
    }

    #[test]
    fn select_account_matches_case_insensitively() {
        let accts = vec![acct("Alice@corp.com", "tid", "oid.tid")];
        let picked = select_account(accts, &cfg("alice@corp.com", None)).unwrap();
        assert_eq!(picked.username, "Alice@corp.com");
    }

    #[test]
    fn select_account_empty_username_takes_first() {
        let accts = vec![acct("a@b.com", "tid", "oid.tid"), acct("c@d.com", "tid2", "oid.tid2")];
        let picked = select_account(accts, &cfg("", None)).unwrap();
        assert_eq!(picked.username, "a@b.com");
    }

    #[test]
    fn select_account_not_found() {
        let accts = vec![acct("bob@corp.com", "tid", "oid.tid")];
        let err = select_account(accts, &cfg("alice@corp.com", None)).unwrap_err();
        assert!(matches!(err, BrokerError::AccountNotFound { .. }));
    }

    #[test]
    fn pick_prefers_explicit_tenant() {
        let matches = vec![
            acct("a@b.com", "home-tid", "oid.home-tid"),
            acct("a@b.com", "guest-tid", "oid.home-tid"),
        ];
        let picked = pick_preferred_realm(matches, Some("guest-tid"));
        assert_eq!(picked.realm, "guest-tid");
    }

    #[test]
    fn pick_prefers_home_tenant_from_home_account_id() {
        let matches = vec![
            acct("a@b.com", "guest-tid", "oid.home-tid"),
            acct("a@b.com", "home-tid", "oid.home-tid"),
        ];
        let picked = pick_preferred_realm(matches, None);
        assert_eq!(picked.realm, "home-tid");
    }

    #[test]
    fn parse_token_response_error() {
        let json = r#"{"brokerTokenResponse":{"error":{"context":"bad","status":3,"subStatus":5}}}"#;
        let err = parse_token_response(json).unwrap_err();
        assert!(matches!(
            err,
            BrokerError::BrokerStatus { status: 3, sub_status: 5, .. }
        ));
    }

    #[test]
    fn token_expiry_prefers_jwt_exp() {
        let jwt = jwt_with(r#"{"exp":1900000000,"tid":"t"}"#);
        let tok = BrokerToken::from_token(jwt, None);
        assert_eq!(tok.expires_on, DateTime::from_timestamp(1_900_000_000, 0).unwrap());
    }

    #[test]
    fn token_expiry_falls_back_when_no_jwt() {
        let fb = Utc::now() + chrono::Duration::seconds(1800);
        let tok = BrokerToken::from_token("opaque".into(), Some(fb));
        assert_eq!(tok.expires_on, fb);
        // A bogus (near-zero) fallback is pinned ~1h out instead.
        let tok2 = BrokerToken::from_token("opaque".into(), Some(Utc::now()));
        assert!(tok2.expires_on > Utc::now() + chrono::Duration::minutes(50));
    }

    #[test]
    fn token_tenant_extracts_tid() {
        let jwt = jwt_with(r#"{"tid":"70a036f6-8e4d-4615-bad6-149c02e7720d","exp":1}"#);
        assert_eq!(
            token_tenant(&jwt).as_deref(),
            Some("70a036f6-8e4d-4615-bad6-149c02e7720d")
        );
        assert_eq!(token_tenant("opaque"), None);
    }

    #[test]
    fn request_serialization_uses_camel_case() {
        let a = acct("a@b.com", "tid", "oid.tid");
        let c = cfg("a@b.com", None);
        let req = AcquireTokenRequest {
            account: &a,
            auth_parameters: auth_parameters(&a, &c, 1),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"authParameters\""));
        assert!(json.contains("\"clientId\":\"cid\""));
        assert!(json.contains("\"authorizationType\":1"));
        assert!(json.contains("\"uxContextHandle\":-1"));
    }

    // Exercises the real broker when one is running; skips cleanly elsewhere.
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    #[test]
    fn live_broker_lists_accounts_when_available() {
        if !available() {
            eprintln!("skipping live broker test: broker not available");
            return;
        }
        // Note: on Windows the WAM AAD *provider* is present on every machine
        // (including a non-joined CI runner), so `available()` can be true with
        // zero registered accounts. The Linux D-Bus broker only advertises
        // itself when accounts exist. So only assert that enumeration succeeds
        // and any returned accounts are well-formed — not that there is >=1.
        let accounts = list_accounts("http://localhost").expect("list accounts");
        for a in &accounts {
            assert!(!a.username.is_empty(), "broker account is missing a username");
        }
    }
}
