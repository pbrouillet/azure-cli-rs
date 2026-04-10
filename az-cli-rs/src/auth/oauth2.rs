/// OAuth2 types: token responses, PKCE, ID token claims.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Azure CLI's well-known public client application ID.
/// From Python auth/constants.py:6
pub const AZURE_CLI_CLIENT_ID: &str = "04b07795-8ddb-461a-bbee-02f9e1bf7b46";

/// Token response from the Microsoft Entra ID token endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    pub token_type: String,
    /// Seconds until expiry (from the token endpoint response).
    pub expires_in: u64,
    #[serde(default)]
    pub scope: Option<String>,
    /// Computed absolute expiry time (not in the raw response).
    #[serde(skip)]
    pub expires_on: DateTime<Utc>,
    /// Decoded claims from the id_token (computed after deserialization).
    #[serde(skip)]
    pub id_token_claims: Option<IdTokenClaims>,
}

impl TokenResponse {
    /// Finalize the response by computing `expires_on` and decoding `id_token`.
    pub fn finalize(mut self) -> Self {
        self.expires_on = Utc::now() + chrono::Duration::seconds(self.expires_in as i64);
        if let Some(ref id_token) = self.id_token {
            self.id_token_claims = decode_id_token(id_token);
        }
        self
    }
}

/// Minimal claims we extract from the ID token (JWT payload).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdTokenClaims {
    #[serde(default)]
    pub preferred_username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    /// Object ID of the user
    #[serde(default)]
    pub oid: Option<String>,
    /// Tenant ID
    #[serde(default)]
    pub tid: Option<String>,
    /// Subject
    #[serde(default)]
    pub sub: Option<String>,
}

/// Decode a JWT's payload (no signature verification — we trust the token endpoint).
fn decode_id_token(token: &str) -> Option<IdTokenClaims> {
    use base64::Engine;
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .ok()?;
    serde_json::from_slice(&payload).ok()
}

/// Generate a PKCE code verifier and challenge.
pub fn generate_pkce() -> (String, String) {
    use base64::Engine;
    use rand::Rng;
    use sha2::{Digest, Sha256};

    let mut rng = rand::thread_rng();
    let verifier_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&verifier_bytes);

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());

    (verifier, challenge)
}

/// Error response from the Microsoft Entra ID token endpoint.
/// Matches the JSON shape: `{ "error": "...", "error_description": "...", "error_codes": [...] }`
#[derive(Debug, Clone, Deserialize)]
pub struct AadErrorResponse {
    pub error: String,
    #[serde(default)]
    pub error_description: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub error_codes: Option<Vec<u64>>,
}

impl AadErrorResponse {
    /// Try to parse an AAD error from a response body.
    pub fn from_body(body: &str) -> Option<Self> {
        serde_json::from_str(body).ok()
    }

    /// Extract a user-friendly error message (the description, or fall back to the error code).
    pub fn message(&self) -> String {
        self.error_description
            .clone()
            .unwrap_or_else(|| self.error.clone())
    }
}

/// Build a suggested re-login command, matching the Python CLI format from auth/util.py:64-89.
///
/// Output example:
/// ```text
/// azrs logout
/// azrs login --tenant "16b3c013-..." --scope "https://management.core.windows.net//.default"
/// ```
pub fn generate_login_suggestion(tenant: Option<&str>, scopes: &[String]) -> String {
    let mut login_parts = vec!["azrs login".to_string()];
    if let Some(t) = tenant {
        login_parts.push(format!("--tenant \"{t}\""));
    }
    if !scopes.is_empty() {
        login_parts.push("--scope".to_string());
        for s in scopes {
            login_parts.push(format!("\"{s}\""));
        }
    }
    format!("azrs logout\n{}", login_parts.join(" "))
}
