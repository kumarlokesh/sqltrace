//! Integration tests for the database module

mod test_utils;

use sqltrace_rs::db::{models::PlanNode, Database};
use sqltrace_rs::SqlTraceError;
use test_utils::with_test_database;

#[tokio::test]
async fn test_explain_simple_query() -> anyhow::Result<()> {
    with_test_database(|pool| async move {
        let db = Database::from_pool(pool);
        let plan = db.explain("SELECT * FROM users WHERE id = 1").await?;

        assert!(!plan.root.node_type.is_empty(), "Expected a valid plan");
        assert!(
            plan.planning_time >= 0.0,
            "Expected planning time to be recorded"
        );
        assert!(
            plan.execution_time >= 0.0,
            "Expected execution time to be recorded"
        );

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_explain_with_join() -> anyhow::Result<()> {
    with_test_database(|pool| async move {
        let db = Database::from_pool(pool);
        let plan = db
            .explain(
                "SELECT u.name, p.title FROM users u 
                JOIN posts p ON u.id = p.user_id 
                WHERE p.published = true",
            )
            .await?;

        // Should have at least one node with a join
        let has_join = plan.root.node_type.contains("Join")
            || plan.root.plans.iter().any(|p| p.node_type.contains("Join"));
        assert!(has_join, "Expected a join in the execution plan");

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_validate_query_rejects_non_select() -> anyhow::Result<()> {
    with_test_database(|pool| async move {
        let db = Database::from_pool(pool);

        // Should reject INSERT
        let result = db
            .explain("INSERT INTO users (name, email) VALUES ('test', 'test@test.com')")
            .await;
        assert!(
            matches!(result, Err(SqlTraceError::InvalidQuery(_))),
            "Expected invalid query error for INSERT"
        );

        // Should reject UPDATE
        let result = db
            .explain("UPDATE users SET name = 'test' WHERE id = 1")
            .await;
        assert!(
            matches!(result, Err(SqlTraceError::InvalidQuery(_))),
            "Expected invalid query error for UPDATE"
        );

        // Should reject DELETE
        let result = db.explain("DELETE FROM users WHERE id = 1").await;
        assert!(
            matches!(result, Err(SqlTraceError::InvalidQuery(_))),
            "Expected invalid query error for DELETE"
        );

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_parse_plan_json_structure() -> anyhow::Result<()> {
    // This test verifies that we can parse the plan JSON structure correctly
    // It uses a static JSON string that matches what we expect from PostgreSQL
    let plan_json = r#"
    [
        {
            "Execution Time": 0.117,
            "Plan": {
                "Node Type": "Sort",
                "Startup Cost": 0.0,
                "Total Cost": 0.0,
                "Plan Rows": 2,
                "Actual Rows": 2,
                "Actual Loops": 1,
                "Actual Startup Time": 0.0,
                "Actual Total Time": 0.0,
                "Plans": [
                    {
                        "Node Type": "Seq Scan",
                        "Relation Name": "users",
                        "Alias": "u",
                        "Startup Cost": 0.0,
                        "Total Cost": 0.0,
                        "Plan Rows": 2,
                        "Plan Width": 68,
                        "Actual Rows": 2,
                        "Actual Loops": 1,
                        "Actual Startup Time": 0.0,
                        "Actual Total Time": 0.0
                    }
                ]
            },
            "Planning Time": 0.723
        }
    ]"#;

    // Parse the JSON into a serde_json::Value
    let plan_value: serde_json::Value =
        serde_json::from_str(plan_json).expect("Failed to parse test plan JSON");

    // Try to parse as a Vec<ExplainPlan>
    let explain_plans: Vec<sqltrace_rs::db::models::plan::ExplainPlan> =
        serde_json::from_value(plan_value).expect("Failed to parse plan as Vec<ExplainPlan>");

    assert!(!explain_plans.is_empty(), "Expected at least one plan");
    assert_eq!(
        explain_plans[0].plan.node_type, "Sort",
        "Expected Sort node at root"
    );
    assert!(
        !explain_plans[0].plan.plans.is_empty(),
        "Expected at least one child plan"
    );
    assert_eq!(
        explain_plans[0].plan.plans[0].node_type, "Seq Scan",
        "Expected Seq Scan as child"
    );

    Ok(())
}

#[tokio::test]
async fn test_explain_with_complex_query() -> anyhow::Result<()> {
    with_test_database(|pool| async move {
        let db = Database::from_pool(pool);

        // Test a more complex query with subquery and aggregation
        let plan = db
            .explain(
                "SELECT u.name, 
                        (SELECT COUNT(*) FROM posts p WHERE p.user_id = u.id) as post_count 
                 FROM users u 
                 WHERE EXISTS (
                     SELECT 1 FROM posts p 
                     WHERE p.user_id = u.id AND p.published = true
                 )",
            )
            .await?;

        assert!(!plan.root.node_type.is_empty(), "Expected a non-empty plan");

        /// Recursively searches for a subplan in the execution plan.
        ///
        /// A subplan is identified by the presence of either:
        /// 1. A "Subplan Name" field that is not null, or
        /// 2. A "Parent Relationship" field with the value "SubPlan"
        ///
        /// Returns a tuple where the first element indicates if a subplan was found,
        /// and the second element contains a string representation of the plan traversal
        /// for debugging purposes.
        fn has_subplan(node: &PlanNode, depth: usize) -> (bool, String) {
            let indent = "  ".repeat(depth);
            let debug_output = format!("{}Node: {}\n", indent, node.node_type);
            if let serde_json::Value::Object(map) = &node.extra {
                // Check if this node is a subplan
                let is_subplan = map.get("Subplan Name").map_or(false, |v| !v.is_null()) ||
                               map.get("Parent Relationship").map_or(false, |v| v == "SubPlan");
                if is_subplan {
                    return (true, debug_output);
                }
                // Recursively check child plans in the extra field
                if let Some(serde_json::Value::Array(plans)) = map.get("Plans") {
                    for plan in plans {
                        if let Ok(child_node) = serde_json::from_value::<PlanNode>(plan.clone()) {
                            let (found, _) = has_subplan(&child_node, depth + 1);
                            if found {
                                return (true, debug_output);
                            }
                        }
                    }
                }
            }
            // Check top-level plans vector
            for child in &node.plans {
                let (found, _) = has_subplan(child, depth + 1);
                if found {
                    return (true, debug_output);
                }
            }
            (false, debug_output)
        }

        // Check for join operation (can be various types like Hash Join, Nested Loop, etc.)
        let has_join = plan.root.node_type.contains("Join");
        // Check for subplan anywhere in the plan tree
        let (has_subplan, _) = has_subplan(&plan.root, 0);

        assert!(
            has_join,
            "Expected plan to include a join operation. This might indicate an issue with the test query or database state."
        );

        assert!(
            has_subplan,
            "Expected plan to include a subplan for the subquery. This might indicate that PostgreSQL optimized the query differently than expected."
        );

        Ok(())
    })
    .await
}
