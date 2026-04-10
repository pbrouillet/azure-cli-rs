/// Service principal authentication — client credentials OAuth2 flow.
///
/// Used for CI/CD and automation scenarios.
/// Supports client_secret authentication (certificate auth is a future enhancement).
use crate::auth::oauth2::TokenResponse;
use crate::error::{AzrsError, Result};

/// Perform service principal login with client secret.
///
/// Uses the OAuth2 client_credentials grant:
/// POST {authority}/oauth2/v2.0/token
///   grant_type=client_credentials
///   client_id=<app_id>
///   client_secret=<secret>
///   scope=<scope>
pub async fn login_with_secret(
    authority: &str,
    client_id: &str,
    client_secret: &str,
    scopes: &[String],
) -> Result<TokenResponse> {
    let token_url = format!("{authority}/oauth2/v2.0/token");
    let scope_str = scopes.join(" ");

    let client = reqwest::Client::new();
    let resp = client
        .post(&token_url)
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "client_credentials"),
            ("scope", &scope_str),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AzrsError::Auth(format!(
            "Service principal login failed: {body}"
        )));
    }

    let token: TokenResponse = resp.json().await?;
    Ok(token.finalize())
}

/// A stored service principal entry (persisted to disk).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpEntry {
    pub client_id: String,
    pub tenant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
}

/// Store for service principal entries — persisted at `~/.azure/azrs_sp_entries.json`.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SpStore {
    pub entries: Vec<SpEntry>,
}

impl SpStore {
    fn path() -> std::path::PathBuf {
        let mut p = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        p.push(".azure");
        p.push("azrs_sp_entries.json");
        p
    }

    /// Load SP store from disk.
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save SP store to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, data)?;
        Ok(())
    }

    /// Add or update an entry.
    pub fn upsert(&mut self, entry: SpEntry) {
        self.entries
            .retain(|e| !(e.client_id == entry.client_id && e.tenant == entry.tenant));
        self.entries.push(entry);
    }

    /// Remove an entry by client_id.
    #[allow(dead_code)]
    pub fn remove(&mut self, client_id: &str) {
        self.entries.retain(|e| e.client_id != client_id);
    }

    /// Find an entry by client_id.
    #[allow(dead_code)]
    pub fn find(&self, client_id: &str) -> Option<&SpEntry> {
        self.entries.iter().find(|e| e.client_id == client_id)
    }
}
