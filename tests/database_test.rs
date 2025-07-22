//! Integration tests for the database module

mod test_utils;

use sqltrace_rs::db::{models::PlanNode, Database};
use sqltrace_rs::SqlTraceError;
use test_utils::with_test_database;

#[tokio::test]
async fn test_explain_simple_query() -> anyhow::Result<()> {
    with_test_database(|pool| async move {
        let db = Database::from_pool(pool);

        // Test a simple query
        let plan = db.explain("SELECT * FROM users WHERE id = 1").await?;

        // Verify we got a plan back
        assert!(!plan.root.node_type.is_empty(), "Expected a valid plan");

        // Verify timing information
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

        // Test a join query
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

        // Debug: Print the plan structure
        println!("Plan structure: {:#?}", plan);

        // Debug function to print the full plan structure
        fn print_plan_node(node: &PlanNode, indent: usize) {
            let indent_str = "  ".repeat(indent);
            let parent_rel = node
                .extra
                .get("Parent Relationship")
                .and_then(|v| v.as_str())
                .unwrap_or("None");

            // Print node type and parent relationship
            println!(
                "{}- {} (Parent Relationship: {})",
                indent_str, node.node_type, parent_rel
            );

            // Print additional node information
            println!("{}  Alias: {:?}", indent_str, node.alias);

            // Print any subplan or subquery information
            if let Some(subplan_name) = node.extra.get("Subplan Name").and_then(|v| v.as_str()) {
                println!("{}  Subplan Name: {}", indent_str, subplan_name);
            }

            if let Some(plan_rows) = node.extra.get("Plan Rows").and_then(|v| v.as_f64()) {
                println!("{}  Plan Rows: {}", indent_str, plan_rows);
            }

            // Print all extra fields for debugging
            if !node.extra.is_null() {
                println!("{}  Extra fields:", indent_str);
                for (key, value) in node.extra.as_object().unwrap() {
                    println!("{}    {}: {:?}", indent_str, key, value);
                }
            }

            // Recursively print child nodes
            for child in &node.plans {
                print_plan_node(child, indent + 1);
            }
        }

        // Print the full plan structure for debugging
        println!("Full execution plan structure:");
        print_plan_node(&plan.root, 0);

        // Debug: Print the full structure of the first few nodes to understand the data structure
        println!("\nDebugging PlanNode structure:");
        println!("Root node type: {}", plan.root.node_type);
        println!("Root node extra fields: {:?}", plan.root.extra);

        if !plan.root.plans.is_empty() {
            println!("\nFirst child node:");
            println!("  Type: {}", plan.root.plans[0].node_type);
            println!("  Extra: {:?}", plan.root.plans[0].extra);

            if !plan.root.plans[0].plans.is_empty() {
                println!("\nFirst grandchild node:");
                println!("  Type: {}", plan.root.plans[0].plans[0].node_type);
                println!("  Extra: {:?}", plan.root.plans[0].plans[0].extra);
            }
        }

        // Check for subplan in the execution plan
        // PostgreSQL represents subqueries as nodes with Parent Relationship "SubPlan" or with a Subplan Name
        fn has_subplan_node(node: &PlanNode) -> bool {
            // Debug: Print the node type and extra fields
            println!("\nChecking node: {}", node.node_type);
            println!("Extra fields: {:?}", node.extra);

            // Check if this node is a SubPlan by checking the extra fields
            let mut is_subplan = false;

            // Check for Parent Relationship = "SubPlan"
            if let Some(serde_json::Value::String(parent_rel)) =
                node.extra.get("Parent Relationship")
            {
                println!("  Found Parent Relationship: {}", parent_rel);
                if parent_rel == "SubPlan" {
                    is_subplan = true;
                    println!("  Found SubPlan via Parent Relationship");
                }
            }

            // Check for Subplan Name
            if node.extra.get("Subplan Name").is_some() {
                println!("  Found Subplan Name");
                is_subplan = true;
            }

            // Check if any child node is a SubPlan
            println!("  Checking {} child nodes...", node.plans.len());
            let has_subplan_in_children = node.plans.iter().any(has_subplan_node);

            is_subplan || has_subplan_in_children
        }

        // Check if any node in the plan is a SubPlan
        let has_subplan = has_subplan_node(&plan.root);

        // Print a message about what we found
        if has_subplan {
            println!("Found a subplan in the execution plan!");
        } else {
            println!("No subplan found in the execution plan based on current detection logic.");

            // Print a more detailed error message to help with debugging
            println!("\nExpected to find a node with either:");
            println!("1. Parent Relationship = 'SubPlan', or");
            println!("2. A 'Subplan Name' field");
            println!("\nBut no such node was found in the execution plan.");
        }

        // Assert that we found a subplan
        assert!(has_subplan, "Expected a subplan in the execution plan");

        Ok(())
    })
    .await
}
