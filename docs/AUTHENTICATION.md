# Authentication

`azrs` implements its own OAuth2 client rather than depending on `azure_identity` or MSAL.
It authenticates as Azure CLI's well-known **public client application**:

```
client_id = 04b07795-8ddb-461a-bbee-02f9e1bf7b46
```

(defined as `AZURE_CLI_CLIENT_ID` in `src/auth/oauth2.rs`). All auth code lives under
`src/auth/`; the shared OAuth2 types and token-endpoint calls are in `oauth2.rs`.

## The six login flows

All flows are entry points of `azrs login`, selected by flags on `LoginArgs`
(`src/cli.rs`), and dispatched in `src/main.rs`. Each obtains an access token (and,
where applicable, a refresh token) and then runs tenant/subscription discovery.

| Flow | Trigger | Source module |
|------|---------|---------------|
| Interactive browser (auth-code + PKCE) | `azrs login` (default) | `auth/interactive.rs` |
| Device code | `azrs login --use-device-code` | `auth/device_code.rs` |
| Service principal (secret) | `azrs login --service-principal -u <appId> -p <secret> --tenant <t>` | `auth/service_principal.rs` |
| Service principal (certificate) | `azrs login --service-principal -u <appId> --certificate <file> [--certificate-password <pw>] --tenant <t>` | `auth/certificate.rs` |
| Managed identity | `azrs login --identity [--client-id/--object-id/--resource-id <id>]` | `auth/managed_identity.rs` |
| Cloud Shell | auto-detected (see below) | `auth/cloud_shell.rs` |

### Interactive browser (default)

Authorization-code flow with PKCE. `azrs` starts a local `tiny_http` redirect server,
opens the system browser to the Entra ID authorize endpoint, receives the auth code on
the loopback redirect, and exchanges it (plus the PKCE verifier) for tokens. When no
`--tenant` is given, the authority defaults to the `organizations` tenant.

### Device code

Requests a device/user code from Entra ID, prints instructions
(`https://microsoft.com/devicelogin` + code) to stderr, and polls the token endpoint until
the user completes sign-in. Useful for headless environments.

### Service principal — client secret

Client-credentials grant using the app ID (`-u/--username`) and secret (`-p/--password`)
against the specified `--tenant`. The SP entry is persisted (see
[SP entry store](#service-principal-entry-store)) so subsequent commands can re-acquire
tokens non-interactively.

### Service principal — certificate

Client-credentials grant using a certificate instead of a secret. `--certificate` points
at the key/cert file; PEM (PKCS#8 private key) and PFX/PKCS#12 (unlocked with
`--certificate-password`) are supported. A signed client assertion (JWT) is sent to the
token endpoint.

### Managed identity

For Azure-hosted environments. `auth/managed_identity.rs` tries the **App Service** identity
endpoint first (using the `IDENTITY_ENDPOINT` + `IDENTITY_HEADER` environment variables),
and otherwise falls back to **IMDS** at
`http://169.254.169.254/metadata/identity/oauth2/token`. User-assigned identities can be
selected with `--client-id`, `--object-id`, or `--resource-id`.

### Cloud Shell

Detected automatically from the environment (the MSI endpoint exposed inside Azure Cloud
Shell) and handled by `auth/cloud_shell.rs`, so `azrs` picks up the ambient identity
without explicit flags.

## OAuth2 scopes

Scopes are derived from the active cloud, matching Python `az` exactly.

- `resource_to_scope(resource)` (`src/cloud.rs`) appends `/.default`:
  `scope = "{resource}/.default"`.
- The default ARM scope is `cloud.default_scope()`, i.e.
  `active_directory_resource_id + "/.default"`. For public cloud this is
  `https://management.core.windows.net/` + `/.default` =
  **`https://management.core.windows.net//.default`**. The double slash is intentional and
  matches `az`.
- Data-plane services override the scope. Key Vault, for instance, uses
  `https://vault.azure.net/.default`.

Per-cloud endpoints (`CloudConfig` in `src/cloud.rs`):

| Cloud | active_directory | active_directory_resource_id |
|-------|------------------|------------------------------|
| Public | `https://login.microsoftonline.com` | `https://management.core.windows.net/` |
| China | `https://login.chinacloudapi.cn` | `https://management.core.chinacloudapi.cn/` |
| US Gov | `https://login.microsoftonline.us` | `https://management.core.usgovcloudapi.net/` |

## Token cache

Persisted at `~/.azure/azrs_token_cache.json` (`src/auth/token_cache.rs`,
`TokenCache`). It stores:

- **Access tokens**, keyed by `"{username}|{tenant}|{scope}"` (all lowercased).
- **Refresh tokens**, keyed by `"{username}|{tenant}"`.

`get_access_token()` returns a cached token when valid, otherwise exchanges the stored
refresh token for a new one via `refresh_access_token()`. `username` and `tenant` are
taken from the ID-token claims (`preferred_username`, tenant id) on the token response.

This cache is `azrs`-specific and separate from the MSAL cache that Python `az` maintains.

## Profile and subscriptions

After a successful login, `src/arm.rs` runs discovery: it calls ARM `/tenants` and
`/subscriptions`, acquiring a per-tenant token from the refresh token for each tenant, and
writes the results to the profile.

- **Profile** — `~/.azure/azureProfile.json` (`src/profile.rs`), the **same format** as
  Python `az`, listing subscriptions and the active one. `azrs` and `az` can share it.
- In a TTY, an interactive subscription picker (`src/selector.rs`) is shown to choose the
  active subscription; `--allow-no-subscriptions` lets login succeed even when none are
  found.

### Service principal entry store

SP credentials are recorded in `~/.azure/azrs_sp_entries.json` (`SpStore` in
`src/auth/service_principal.rs`) so non-interactive re-authentication works across command
invocations.

## Re-login suggestions

When a call fails in a way that a re-login would fix (wrong tenant or scope), commands
return `AzrsError::AuthWithSuggestion { message, suggestion }`. `main.rs` prints the
`suggestion` to stderr — a ready-to-run command such as:

```text
azrs logout
azrs login --tenant "16b3c013-…" --scope "https://management.core.windows.net//.default"
```

See `generate_login_suggestion()` in `src/auth/oauth2.rs`.

## Related docs

- [ARCHITECTURE.md](ARCHITECTURE.md) — how auth plugs into the ARM command framework.
- [../README.md](../README.md) — quickstart and state-file overview.
