/// Profile persistence — manages `~/.azure/azureProfile.json`.
///
/// Mirrors the profile format from Python azure-cli _profile.py.
use crate::auth::oauth2::IdTokenClaims;
use crate::error::{AzrsError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level profile stored in azureProfile.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    #[serde(default = "generate_installation_id")]
    pub installation_id: String,
    #[serde(default)]
    pub subscriptions: Vec<Subscription>,
}

fn generate_installation_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// A single subscription entry (matches Python _profile.py:416-443).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: String,
    pub name: String,
    pub state: String,
    pub tenant_id: String,
    #[serde(default)]
    pub is_default: bool,
    pub environment_name: String,
    pub user: SubscriptionUser,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_default_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_by_tenants: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionUser {
    pub name: String,
    #[serde(rename = "type")]
    pub user_type: String,
}

impl Profile {
    /// Path to the profile file.
    fn path() -> PathBuf {
        let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        p.push(".azure");
        p.push("azureProfile.json");
        p
    }

    /// Load profile from disk (or create empty if it doesn't exist).
    pub fn load() -> Result<Self> {
        let path = Self::path();
        if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            // Handle BOM that az python sometimes writes
            let data = data.trim_start_matches('\u{feff}');
            let profile: Profile = serde_json::from_str(data).unwrap_or_else(|_| Profile {
                installation_id: generate_installation_id(),
                subscriptions: Vec::new(),
            });
            Ok(profile)
        } else {
            Ok(Profile {
                installation_id: generate_installation_id(),
                subscriptions: Vec::new(),
            })
        }
    }

    /// Save profile to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, data)?;
        Ok(())
    }

    /// Get the active (default) subscription.
    pub fn active_subscription(&self) -> Option<&Subscription> {
        self.subscriptions.iter().find(|s| s.is_default)
    }

    /// Find a subscription by ID or name (case-insensitive name match).
    pub fn find_subscription(&self, id_or_name: &str) -> Option<&Subscription> {
        self.subscriptions.iter().find(|s| {
            s.id == id_or_name || s.name.to_lowercase() == id_or_name.to_lowercase()
        })
    }

    /// Set a subscription as the active one.
    pub fn set_active_subscription(&mut self, id_or_name: &str) -> Result<()> {
        let found = self.subscriptions.iter().any(|s| {
            s.id == id_or_name || s.name.to_lowercase() == id_or_name.to_lowercase()
        });
        if !found {
            return Err(AzrsError::SubscriptionNotFound(id_or_name.to_string()));
        }
        for sub in &mut self.subscriptions {
            sub.is_default =
                sub.id == id_or_name || sub.name.to_lowercase() == id_or_name.to_lowercase();
        }
        Ok(())
    }

    /// Merge newly discovered subscriptions into the profile.
    /// Sets the first subscription as default if none is currently default.
    pub fn merge_subscriptions(
        &mut self,
        new_subs: Vec<Subscription>,
        _id_claims: &Option<IdTokenClaims>,
        _env_name: &str,
    ) {
        // Remove existing subs that match the new ones by ID
        let new_ids: Vec<&str> = new_subs.iter().map(|s| s.id.as_str()).collect();
        self.subscriptions
            .retain(|s| !new_ids.contains(&s.id.as_str()));
        self.subscriptions.extend(new_subs);

        // If no default is set, pick the first one
        if !self.subscriptions.iter().any(|s| s.is_default) {
            if let Some(first) = self.subscriptions.first_mut() {
                first.is_default = true;
            }
        }
    }

    /// Remove all subscriptions for a given username.
    pub fn remove_subscriptions_for_user(&mut self, username: &str) {
        self.subscriptions
            .retain(|s| s.user.name.to_lowercase() != username.to_lowercase());
        // Re-elect a default if the current one was removed
        if !self.subscriptions.iter().any(|s| s.is_default) {
            if let Some(first) = self.subscriptions.first_mut() {
                first.is_default = true;
            }
        }
    }
}
