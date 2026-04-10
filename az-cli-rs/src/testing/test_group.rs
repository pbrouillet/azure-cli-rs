/// Integration tests for resource group operations using cassette playback.
///
/// These tests exercise ArmCommand methods (get/put/list/delete/exists) against
/// recorded HTTP interactions, verifying the ARM framework works correctly.
#[cfg(test)]
mod group_tests {
    use crate::testing::checkers;
    use crate::testing::scenario::{cmd_result_from_json, ScenarioTest};

    #[tokio::test]
    async fn test_group_crud() {
        let mut test = ScenarioTest::new("group", "test_group_crud");
        let rg_name = test.create_random_name("clitest.rg", 24);
        let mut cmd = test.arm_command();

        // CREATE — PUT a resource group
        let create_path = format!(
            "/subscriptions/{{subscriptionId}}/resourcegroups/{rg_name}?api-version=2024-03-01"
        );
        let body = serde_json::json!({ "location": "eastus", "tags": {} });
        let result = cmd.put(&create_path, &body).await.expect("create should succeed");
        let cr = cmd_result_from_json(result);
        cr.assert_with_checks(&[
            test.check("name", serde_json::json!(rg_name)),
            test.check("location", serde_json::json!("eastus")),
            test.exists("id"),
            test.check("properties.provisioningState", serde_json::json!("Succeeded")),
        ]);

        // SHOW — GET the resource group
        let show_path = format!(
            "/subscriptions/{{subscriptionId}}/resourcegroups/{rg_name}?api-version=2024-03-01"
        );
        let result = cmd.get(&show_path).await.expect("show should succeed");
        let cr = cmd_result_from_json(result);
        cr.assert_with_checks(&[
            test.check("name", serde_json::json!(rg_name)),
            test.check("location", serde_json::json!("eastus")),
        ]);

        // LIST — list all resource groups
        let list_path =
            "/subscriptions/{subscriptionId}/resourcegroups?api-version=2024-03-01";
        let results = cmd.list(list_path).await.expect("list should succeed");
        assert!(!results.is_empty(), "list should return at least one group");
        let list_json = serde_json::Value::Array(results);
        let cr = cmd_result_from_json(list_json);
        cr.assert_with_checks(&[
            checkers::greater_than("length(@)", 0.0),
            test.check("[0].name", serde_json::json!(rg_name)),
        ]);

        // EXISTS — HEAD returns 204 (true)
        let exists_path = format!(
            "/subscriptions/{{subscriptionId}}/resourcegroups/{rg_name}?api-version=2024-03-01"
        );
        let exists = cmd.exists(&exists_path).await.expect("exists should succeed");
        assert!(exists, "resource group should exist");

        // DELETE — returns 202
        let delete_path = format!(
            "/subscriptions/{{subscriptionId}}/resourcegroups/{rg_name}?api-version=2024-03-01"
        );
        cmd.delete(&delete_path).await.expect("delete should succeed");

        // EXISTS after delete — should be false (404)
        let exists = cmd.exists(&exists_path).await.expect("exists should succeed");
        assert!(!exists, "resource group should not exist after deletion");

        test.finish();
    }
}
