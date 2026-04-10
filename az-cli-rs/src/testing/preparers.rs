/// Resource preparers — create and clean up Azure resources for tests.
///
/// Inspired by Python testsdk's `preparers.py`.
/// Preparers create resources before the test runs and clean them up after,
/// even if the test panics.
use super::scenario::ScenarioTest;

/// Trait for resource preparers.
pub trait Preparer: Send {
    /// Create the resource, returning key=value pairs passed to the test.
    fn setup(
        &mut self,
        ctx: &mut ScenarioTest,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = crate::error::Result<Vec<(String, String)>>>
                + Send
                + '_,
        >,
    >;

    /// Clean up the resource after the test.
    fn teardown(
        &mut self,
        ctx: &mut ScenarioTest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = crate::error::Result<()>> + Send + '_>,
    >;
}

/// Preparer for resource groups.
pub struct ResourceGroupPreparer {
    pub name_prefix: String,
    pub location: String,
    created_name: Option<String>,
}

impl ResourceGroupPreparer {
    pub fn new() -> Self {
        Self {
            name_prefix: "clitest.rg".to_string(),
            location: "eastus".to_string(),
            created_name: None,
        }
    }

    pub fn with_location(mut self, location: &str) -> Self {
        self.location = location.to_string();
        self
    }

    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.name_prefix = prefix.to_string();
        self
    }
}

impl Preparer for ResourceGroupPreparer {
    fn setup(
        &mut self,
        ctx: &mut ScenarioTest,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = crate::error::Result<Vec<(String, String)>>>
                + Send
                + '_,
        >,
    > {
        let name = ctx.create_random_name(&self.name_prefix, 24);
        let location = self.location.clone();
        self.created_name = Some(name.clone());

        Box::pin(async move {
            // group::create builds its own ArmCommand internally
            crate::commands::group::create(&name, &location, None).await?;
            Ok(vec![
                ("resource_group".to_string(), name),
                ("resource_group_location".to_string(), location),
            ])
        })
    }

    fn teardown(
        &mut self,
        _ctx: &mut ScenarioTest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = crate::error::Result<()>> + Send + '_>,
    > {
        let name = self.created_name.clone();
        Box::pin(async move {
            if let Some(name) = name {
                // Best-effort cleanup
                let _ = crate::commands::group::delete(&name).await;
            }
            Ok(())
        })
    }
}
