/// Test fixtures — fake profile, token cache, and environment for isolated tests.
///
/// Provides pre-built test data so tests don't depend on real `~/.azure/` files.
use crate::auth::token_cache::{CachedRefreshToken, CachedToken, TokenCache};
use crate::cloud::CloudConfig;
use crate::config::Config;
use crate::profile::{Profile, Subscription, SubscriptionUser};

/// Mocked constants matching Python testsdk conventions.
pub const MOCK_SUBSCRIPTION_ID: &str = "00000000-0000-0000-0000-000000000000";
pub const MOCK_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";
pub const MOCK_USERNAME: &str = "test@example.com";
pub const MOCK_ACCESS_TOKEN: &str = "mock-access-token-for-testing";

/// Test environment — provides fake profile, cache, and config for test isolation.
pub struct TestEnv {
    pub cloud: CloudConfig,
    pub profile: Profile,
    pub cache: TokenCache,
    pub config: Config,
}

impl TestEnv {
    /// Create a new test environment with a single mock subscription.
    pub fn new() -> Self {
        let cloud = CloudConfig::default();
        let profile = fake_profile(&[mock_subscription()]);
        let cache = fake_token_cache(&cloud);
        let config = Config::default();
        Self {
            cloud,
            profile,
            cache,
            config,
        }
    }

    /// Create a test environment with custom subscriptions.
    pub fn with_subscriptions(subscriptions: Vec<Subscription>) -> Self {
        let cloud = CloudConfig::default();
        let profile = fake_profile(&subscriptions);
        let cache = fake_token_cache(&cloud);
        let config = Config::default();
        Self {
            cloud,
            profile,
            cache,
            config,
        }
    }
}

/// Build a mock subscription.
pub fn mock_subscription() -> Subscription {
    Subscription {
        id: MOCK_SUBSCRIPTION_ID.to_string(),
        name: "Test Subscription".to_string(),
        state: "Enabled".to_string(),
        tenant_id: MOCK_TENANT_ID.to_string(),
        is_default: true,
        environment_name: "AzureCloud".to_string(),
        user: SubscriptionUser {
            name: MOCK_USERNAME.to_string(),
            user_type: "user".to_string(),
        },
        home_tenant_id: Some(MOCK_TENANT_ID.to_string()),
        tenant_display_name: None,
        tenant_default_domain: None,
        managed_by_tenants: None,
    }
}

/// Build a Profile with the given subscriptions.
pub fn fake_profile(subscriptions: &[Subscription]) -> Profile {
    Profile {
        installation_id: "test-installation-id".to_string(),
        subscriptions: subscriptions.to_vec(),
    }
}

/// Build a TokenCache pre-loaded with a mock access token for the default subscription.
pub fn fake_token_cache(cloud: &CloudConfig) -> TokenCache {
    let mut cache = TokenCache::new_with_cloud(cloud);
    let scope = cloud.default_scope();
    let key = format!(
        "{}|{}|{}",
        MOCK_USERNAME.to_lowercase(),
        MOCK_TENANT_ID.to_lowercase(),
        scope.to_lowercase()
    );
    cache.access_tokens.insert(
        key,
        CachedToken {
            access_token: MOCK_ACCESS_TOKEN.to_string(),
            expires_on: chrono::Utc::now() + chrono::Duration::hours(1),
            scope,
        },
    );

    let refresh_key = format!(
        "{}|{}",
        MOCK_USERNAME.to_lowercase(),
        MOCK_TENANT_ID.to_lowercase()
    );
    cache.refresh_tokens.insert(
        refresh_key,
        CachedRefreshToken {
            refresh_token: "mock-refresh-token".to_string(),
            username: MOCK_USERNAME.to_string(),
            tenant: MOCK_TENANT_ID.to_string(),
            home_account_id: None,
        },
    );

    cache
}
