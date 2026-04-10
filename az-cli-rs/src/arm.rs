/// ARM REST client for tenant and subscription discovery.
///
/// Mirrors the discovery logic from Python _profile.py:777-876.
use crate::auth::oauth2::TokenResponse;
use crate::auth::TokenCache;
use crate::cloud::CloudConfig;
use crate::error::{AzrsError, Result};
use crate::profile::{Subscription, SubscriptionUser};
use serde::Deserialize;

pub struct ArmClient {
    cloud: CloudConfig,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct ListResponse<T> {
    value: Vec<T>,
    #[serde(rename = "nextLink")]
    next_link: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TenantInfo {
    tenant_id: String,
    display_name: Option<String>,
    default_domain: Option<String>,
    #[allow(dead_code)]
    tenant_type: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubscriptionInfo {
    subscription_id: String,
    display_name: String,
    state: String,
    tenant_id: String,
}

impl ArmClient {
    pub fn new(cloud: &CloudConfig) -> Self {
        Self {
            cloud: cloud.clone(),
            http: reqwest::Client::new(),
        }
    }

    /// GET with Bearer token, following nextLink pagination.
    async fn get_all<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
    ) -> Result<Vec<T>> {
        let mut results = Vec::new();
        let mut next_url = Some(url.to_string());

        while let Some(u) = next_url {
            let resp = self
                .http
                .get(&u)
                .bearer_auth(token)
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(AzrsError::Auth(format!(
                    "ARM request failed ({status}): {body}"
                )));
            }

            let page: ListResponse<T> = resp.json().await?;
            results.extend(page.value);
            next_url = page.next_link;
        }

        Ok(results)
    }

    /// List all tenants accessible with the given token.
    async fn list_tenants(&self, token: &str) -> Result<Vec<TenantInfo>> {
        let url = format!(
            "{}tenants?api-version=2022-12-01",
            self.cloud.resource_manager
        );
        self.get_all(&url, token).await
    }

    /// List subscriptions in a specific tenant.
    async fn list_subscriptions_with_token(
        &self,
        token: &str,
    ) -> Result<Vec<SubscriptionInfo>> {
        let url = format!(
            "{}subscriptions?api-version=2022-12-01",
            self.cloud.resource_manager
        );
        self.get_all(&url, token).await
    }

    /// Discover subscriptions for a single tenant (when --tenant was specified).
    pub async fn discover_subscriptions_for_tenant(
        &self,
        tenant: &str,
        access_token: &str,
    ) -> Result<Vec<Subscription>> {
        let subs = self.list_subscriptions_with_token(access_token).await?;
        Ok(subs
            .into_iter()
            .map(|s| self.to_subscription(s, "user@azure", "user", tenant, None, None))
            .collect())
    }

    /// Discover all tenants and their subscriptions (when no --tenant was specified).
    ///
    /// Uses the initial token to list tenants, then exchanges the refresh token
    /// for per-tenant tokens to list each tenant's subscriptions.
    pub async fn discover_all_subscriptions(
        &self,
        initial_token: &TokenResponse,
        cache: &mut TokenCache,
    ) -> Result<Vec<Subscription>> {
        let username = initial_token
            .id_token_claims
            .as_ref()
            .and_then(|c| c.preferred_username.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // List all tenants
        let tenants = self.list_tenants(&initial_token.access_token).await?;
        let mut all_subs = Vec::new();

        for tenant in &tenants {
            // Get a token for this specific tenant via refresh token
            let tenant_token = if let Some(rt) = &initial_token.refresh_token {
                let scopes = vec![self.cloud.default_scope()];
                match cache
                    .refresh_access_token(rt, &tenant.tenant_id, &scopes, &self.cloud)
                    .await
                {
                    Ok(t) => {
                        cache.store_tokens_for(&t, &username, &tenant.tenant_id)?;
                        t
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Could not acquire token for tenant {} ({}): {e}",
                            tenant.tenant_id,
                            tenant.display_name.as_deref().unwrap_or("unknown")
                        );
                        continue;
                    }
                }
            } else {
                // Fall back to using the initial token (may not work for other tenants)
                initial_token.clone()
            };

            match self
                .list_subscriptions_with_token(&tenant_token.access_token)
                .await
            {
                Ok(subs) => {
                    for sub in subs {
                        all_subs.push(self.to_subscription(
                            sub,
                            &username,
                            "user",
                            &tenant.tenant_id,
                            tenant.display_name.as_deref(),
                            tenant.default_domain.as_deref(),
                        ));
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Could not list subscriptions for tenant {}: {e}",
                        tenant.tenant_id
                    );
                }
            }
        }

        // Deduplicate subscriptions by ID (keep first occurrence).
        // Matches Python _profile.py:814-828 dedup logic.
        let mut seen = std::collections::HashSet::new();
        all_subs.retain(|sub| {
            if seen.contains(&sub.id) {
                eprintln!(
                    "Warning: subscription '{}' ({}) accessible from multiple tenants, keeping first.",
                    sub.name, sub.id
                );
                false
            } else {
                seen.insert(sub.id.clone());
                true
            }
        });

        Ok(all_subs)
    }

    fn to_subscription(
        &self,
        info: SubscriptionInfo,
        username: &str,
        user_type: &str,
        tenant_id: &str,
        tenant_display_name: Option<&str>,
        tenant_default_domain: Option<&str>,
    ) -> Subscription {
        Subscription {
            id: info.subscription_id,
            name: info.display_name,
            state: info.state,
            tenant_id: info.tenant_id.clone(),
            is_default: false,
            environment_name: self.cloud.environment_name.clone(),
            user: SubscriptionUser {
                name: username.to_string(),
                user_type: user_type.to_string(),
            },
            home_tenant_id: Some(tenant_id.to_string()),
            tenant_display_name: tenant_display_name.map(|s| s.to_string()),
            tenant_default_domain: tenant_default_domain.map(|s| s.to_string()),
            managed_by_tenants: Some(Vec::new()),
        }
    }
}
