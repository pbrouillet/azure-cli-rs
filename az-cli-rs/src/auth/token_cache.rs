/// Token cache — stores and retrieves OAuth2 tokens with file persistence.
///
/// Simplified cache that stores tokens keyed by (username, tenant, scope).
/// Supports silent acquisition via refresh tokens.
use crate::auth::oauth2::{AadErrorResponse, TokenResponse, generate_login_suggestion, AZURE_CLI_CLIENT_ID};
use crate::cloud::CloudConfig;
use crate::error::{AzrsError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A cached access token with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedToken {
    pub access_token: String,
    pub expires_on: DateTime<Utc>,
    pub scope: String,
}

/// A cached refresh token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedRefreshToken {
    pub refresh_token: String,
    pub username: String,
    pub tenant: String,
    pub home_account_id: Option<String>,
}

/// Persistent token cache stored at `~/.azure/azrs_token_cache.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenCache {
    /// Access tokens keyed by "{username}|{tenant}|{scope}"
    pub access_tokens: HashMap<String, CachedToken>,
    /// Refresh tokens keyed by "{username}|{tenant}"
    pub refresh_tokens: HashMap<String, CachedRefreshToken>,

    /// Cloud config (not serialized — set on load)
    #[serde(skip)]
    cloud: Option<CloudConfig>,
}

impl TokenCache {
    fn path() -> PathBuf {
        let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        p.push(".azure");
        p.push("azrs_token_cache.json");
        p
    }

    /// Create a new empty cache with a cloud config (for testing).
    pub fn new_with_cloud(cloud: &CloudConfig) -> Self {
        let mut cache = Self::default();
        cache.cloud = Some(cloud.clone());
        cache
    }

    /// Load the cache from disk (or create empty).
    pub fn load(cloud: &CloudConfig) -> Result<Self> {
        let path = Self::path();
        let mut cache = if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        };
        cache.cloud = Some(cloud.clone());
        Ok(cache)
    }

    /// Save the cache to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, data)?;
        Ok(())
    }

    fn access_key(username: &str, tenant: &str, scope: &str) -> String {
        format!(
            "{}|{}|{}",
            username.to_lowercase(),
            tenant.to_lowercase(),
            scope.to_lowercase()
        )
    }

    fn refresh_key(username: &str, tenant: &str) -> String {
        format!("{}|{}", username.to_lowercase(), tenant.to_lowercase())
    }

    /// Store tokens from a login response.
    pub fn store_tokens(&mut self, response: &TokenResponse) -> Result<()> {
        let username = response
            .id_token_claims
            .as_ref()
            .and_then(|c| c.preferred_username.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let tenant = response
            .id_token_claims
            .as_ref()
            .and_then(|c| c.tid.clone())
            .unwrap_or_else(|| "common".to_string());

        let scope = response
            .scope
            .clone()
            .unwrap_or_else(|| "".to_string());

        let home_account_id = response
            .id_token_claims
            .as_ref()
            .and_then(|c| {
                let oid = c.oid.as_deref()?;
                let tid = c.tid.as_deref()?;
                Some(format!("{oid}.{tid}"))
            });

        // Store access token
        let ak = Self::access_key(&username, &tenant, &scope);
        self.access_tokens.insert(
            ak,
            CachedToken {
                access_token: response.access_token.clone(),
                expires_on: response.expires_on,
                scope: scope.clone(),
            },
        );

        // Store refresh token
        if let Some(ref rt) = response.refresh_token {
            let rk = Self::refresh_key(&username, &tenant);
            self.refresh_tokens.insert(
                rk,
                CachedRefreshToken {
                    refresh_token: rt.clone(),
                    username: username.clone(),
                    tenant: tenant.clone(),
                    home_account_id,
                },
            );
        }

        Ok(())
    }

    /// Store tokens for a specific user/tenant (used during subscription discovery).
    pub fn store_tokens_for(
        &mut self,
        response: &TokenResponse,
        username: &str,
        tenant: &str,
    ) -> Result<()> {
        let scope = response.scope.clone().unwrap_or_default();

        let ak = Self::access_key(username, tenant, &scope);
        self.access_tokens.insert(
            ak,
            CachedToken {
                access_token: response.access_token.clone(),
                expires_on: response.expires_on,
                scope: scope.clone(),
            },
        );

        if let Some(ref rt) = response.refresh_token {
            let rk = Self::refresh_key(username, tenant);
            self.refresh_tokens.insert(
                rk,
                CachedRefreshToken {
                    refresh_token: rt.clone(),
                    username: username.to_string(),
                    tenant: tenant.to_string(),
                    home_account_id: None,
                },
            );
        }

        Ok(())
    }

    /// Get a valid access token, refreshing if expired.
    pub async fn get_access_token(
        &mut self,
        username: &str,
        tenant: &str,
        scopes: &[String],
        cloud: &CloudConfig,
    ) -> Result<CachedToken> {
        let scope_str = scopes.join(" ");
        let ak = Self::access_key(username, tenant, &scope_str);

        // Check if we have a valid cached token with exact scope match
        if let Some(cached) = self.access_tokens.get(&ak) {
            if cached.expires_on > Utc::now() + chrono::Duration::minutes(5) {
                return Ok(cached.clone());
            }
        }

        // Try to refresh using tenant-specific refresh token first,
        // then fall back to any refresh token for this user
        let rk = Self::refresh_key(username, tenant);
        let rt = self
            .refresh_tokens
            .get(&rk)
            .map(|r| r.refresh_token.clone())
            .or_else(|| {
                // Fallback: find any refresh token for this user (different tenant)
                let lower = username.to_lowercase();
                self.refresh_tokens
                    .iter()
                    .find(|(k, _)| k.starts_with(&format!("{lower}|")))
                    .map(|(_, v)| v.refresh_token.clone())
            });

        if let Some(refresh_token) = rt {
            let token = self
                .refresh_access_token(&refresh_token, tenant, scopes, cloud)
                .await?;

            let cached = CachedToken {
                access_token: token.access_token.clone(),
                expires_on: token.expires_on,
                scope: scope_str.clone(),
            };
            self.access_tokens.insert(ak, cached.clone());

            if let Some(ref new_rt) = token.refresh_token {
                self.refresh_tokens.insert(
                    rk,
                    CachedRefreshToken {
                        refresh_token: new_rt.clone(),
                        username: username.to_string(),
                        tenant: tenant.to_string(),
                        home_account_id: None,
                    },
                );
            }

            return Ok(cached);
        }

        // No cached/refresh token — try a silent broker acquisition (Linux
        // Identity Broker / Windows WAM). This makes broker-backed logins
        // self-refreshing: a fresh token is minted with no prompt whenever the
        // cached one expires. `available()` is false on unsupported platforms,
        // so this is a no-op there and the suggestion error below still fires.
        if let Some(cached) = self.try_broker_silent(username, tenant, scopes, cloud, &ak, &scope_str) {
            return Ok(cached);
        }

        Err(AzrsError::AuthWithSuggestion {
            message: "Can't find token from cache. To re-authenticate, please run the command below.".into(),
            suggestion: generate_login_suggestion(Some(tenant), scopes),
        })
    }

    /// Attempt a silent broker/WAM token acquisition, caching and returning the
    /// result on success. Returns `None` when the broker is unavailable or the
    /// acquisition fails, letting the caller fall through to its normal error.
    fn try_broker_silent(
        &mut self,
        username: &str,
        tenant: &str,
        scopes: &[String],
        cloud: &CloudConfig,
        access_key: &str,
        scope_str: &str,
    ) -> Option<CachedToken> {
        use crate::auth::broker::{self, BrokerConfig};
        if !broker::available() {
            return None;
        }
        // A concrete GUID tenant disambiguates a multi-realm account; the
        // discovery placeholders (`common`/`organizations`) do not.
        let broker_tenant = (tenant != "common" && tenant != "organizations")
            .then(|| tenant.to_string());
        let authority = format!("{}/{}", cloud.active_directory, tenant);
        let mut cfg = BrokerConfig::for_arm(
            authority,
            scopes.to_vec(),
            username.to_string(),
            broker_tenant,
        );
        let tok = match broker::acquire_token_silent(&cfg) {
            Ok(t) => t,
            Err(_) => {
                // The profile may carry a placeholder username (`user@azure`
                // from single-tenant discovery); retry letting the broker pick
                // its sole/first account.
                cfg.username.clear();
                broker::acquire_token_silent(&cfg).ok()?
            }
        };
        let cached = CachedToken {
            access_token: tok.access_token,
            expires_on: tok.expires_on,
            scope: scope_str.to_string(),
        };
        self.access_tokens.insert(access_key.to_string(), cached.clone());
        Some(cached)
    }

    /// Store an access token acquired out-of-band (e.g. via the broker, which
    /// yields no refresh token) so subsequent calls hit the cache.
    pub fn store_access_token(
        &mut self,
        username: &str,
        tenant: &str,
        scopes: &[String],
        access_token: String,
        expires_on: DateTime<Utc>,
    ) {
        let scope = scopes.join(" ");
        let ak = Self::access_key(username, tenant, &scope);
        self.access_tokens.insert(
            ak,
            CachedToken {
                access_token,
                expires_on,
                scope,
            },
        );
    }

    /// Use a refresh token to get new tokens.
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
        tenant: &str,
        scopes: &[String],
        cloud: &CloudConfig,
    ) -> Result<TokenResponse> {
        let token_url = format!(
            "{}/{}/oauth2/v2.0/token",
            cloud.active_directory, tenant
        );
        let scope_str = scopes.join(" ");

        let client = reqwest::Client::new();
        let resp = client
            .post(&token_url)
            .form(&[
                ("client_id", AZURE_CLI_CLIENT_ID),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("scope", &scope_str),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            let suggestion = generate_login_suggestion(Some(tenant), scopes);

            let message = if let Some(aad_err) = AadErrorResponse::from_body(&body) {
                aad_err.message()
            } else {
                format!("Token refresh failed: {body}")
            };

            return Err(AzrsError::AuthWithSuggestion {
                message,
                suggestion,
            });
        }

        let token: TokenResponse = resp.json().await?;
        Ok(token.finalize())
    }

    /// Remove all tokens for a given user.
    pub fn remove_tokens_for_user(&mut self, username: &str) {
        let lower = username.to_lowercase();
        self.access_tokens
            .retain(|k, _| !k.starts_with(&format!("{lower}|")));
        self.refresh_tokens
            .retain(|k, _| !k.starts_with(&format!("{lower}|")));
    }

    /// Get a refresh token for a specific user (any tenant).
    /// Used during subscription discovery to exchange for tenant-specific tokens.
    #[allow(dead_code)]
    pub fn get_any_refresh_token(&self, username: &str) -> Option<&str> {
        let lower = username.to_lowercase();
        self.refresh_tokens
            .iter()
            .find(|(k, _)| k.starts_with(&format!("{lower}|")))
            .map(|(_, v)| v.refresh_token.as_str())
    }
}
