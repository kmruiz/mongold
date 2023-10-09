use std::cell::RefCell;
use std::error::Error;

use tree_sitter::{Node, Tree};

use mongodb_query_language::execution::{Execution, ExecutionNamespace};
use mongodb_query_language::execution::Execution::FindOne;
use mongodb_query_language::values::Value::Reference;

use crate::tree_ext::infer_mongodb_namespace::infer_mongodb_namespace;
use crate::tree_ext::optional_node_to_string;
use crate::tree_ext::predicate_from_driver_method::predicate_from_driver_method;
use crate::tree_ext::friendly_capture::{FriendlyCapture};

const ALL_FIND_METHOD_CALLS: &str = include_str!("queries/find_one.all_finds.scm");
const ALL_FIND_METHOD_CALLS_ARGUMENT_LIST: &str = include_str!("queries/find_one.all_finds.argument_list.scm");

pub fn find_one(tree: RefCell<Tree>, code: &String) -> Result<Vec<Execution>, Box<dyn Error + Sync + Send>> {
    let all_queries_query = tree_sitter::Query::new(tree_sitter_java::language(), ALL_FIND_METHOD_CALLS)?;
    let all_predicates_query = tree_sitter::Query::new(tree_sitter_java::language(), ALL_FIND_METHOD_CALLS_ARGUMENT_LIST)?;

    let mut cursor = tree_sitter::QueryCursor::new();
    let root = tree.borrow();
    let namespaces = infer_mongodb_namespace(root.root_node(), code)?;
    let all_matches = cursor.matches(&all_queries_query, root.root_node(), code.as_bytes());

    let collection_idx = all_queries_query.capture_index_for_name("collection").unwrap();
    let arglist_idx = all_queries_query.capture_index_for_name("argumentlist").unwrap();
    let method_idx = all_predicates_query.capture_index_for_name("method").unwrap();
    let field_idx = all_predicates_query.capture_index_for_name("field").unwrap();
    let value_idx = all_predicates_query.capture_index_for_name("value").unwrap();

    let mut result: Vec<Execution> = vec![];

    for each_match in all_matches {
        let [ coll_node, arg_list_node ] = &each_match.capture(vec![ collection_idx, arglist_idx ])[..] else { break };
        let mut arg_cursor = tree_sitter::QueryCursor::new();
        let all_predicates = arg_cursor.matches(&all_predicates_query, arg_list_node.unwrap(), code.as_bytes());

        for each_predicate in all_predicates {
            let [
                field_name_node,
                operation_node,
                value_node
            ] = &each_predicate.capture(vec![
                field_idx,
                method_idx,
                value_idx
            ])[..] else { break };

            let coll_field_name = optional_node_to_string(&coll_node, code);
            let query_field_name = optional_node_to_string(&field_name_node, code);
            let operation_name = optional_node_to_string(&operation_node, code);
            let value = Reference(optional_node_to_string(&value_node, code), "any".to_string());

            result.push(FindOne {
                namespace: namespaces.get(&*coll_field_name).map(|x| x.clone()).unwrap_or(ExecutionNamespace::empty(coll_field_name)),
                predicate: predicate_from_driver_method(&operation_name, query_field_name, value)
            });
        }
    }

    return Ok(result);
}

#[cfg(test)]
mod test {
    use mongodb_query_language::execution::Execution::FindOne;
    use mongodb_query_language::execution::ExecutionNamespace;
    use mongodb_query_language::filter::FilterOperator::{Equals, GreaterThan};
    use mongodb_query_language::values::Value::Reference;

    use crate::Java;
    use crate::use_cases::find_one::find_one;

    #[test]
    fn parse_common_find_where_collection_is_a_class_property() {
        let code = r#"
        public class MyRepository {
            private final Collection<Document> collection;

            public Document findOne(String id) {
                return collection.find(eq("_id", id)).first();
            }
        }
        "#.to_string();

        let java = Java::new();
        let tree = java.full_parse(&code);
        let result = find_one(tree, &code).unwrap();

        assert_eq!(result.len(), 1);
        let first = &result[0];

        assert_eq!(*first, FindOne {
            namespace: ExecutionNamespace {
                collection: None,
                database: None,
                reference_name: "collection".to_string()
            },
            predicate: Equals {
                field: "_id".to_string(),
                value: Reference("id".to_string(), "any".to_string())
            }
        })
    }

    #[test]
    fn parse_namespace_from_javadoc_in_field() {
        let code = r#"
        public class MyRepository {
            /**
              * @mongodb.namespace mydb.mycoll
            **/
            private final Collection<Document> myMongoCollection;

            public Document findOne(String id) {
                return myMongoCollection.find(eq("_id", id)).first();
            }
        }
        "#.to_string();

        let java = Java::new();
        let tree = java.full_parse(&code);
        let result = find_one(tree, &code).unwrap();

        assert_eq!(result.len(), 1);
        let first = &result[0];

        assert_eq!(*first, FindOne {
            namespace: ExecutionNamespace {
                database: Some("mydb".to_string()),
                collection: Some("mycoll".to_string()),
                reference_name: "myMongoCollection".to_string()
            },
            predicate: Equals {
                field: "_id".to_string(),
                value: Reference("id".to_string(), "any".to_string())
            }
        })
    }

    #[test]
    fn parse_gt_query() {
        let code = r#"
        public class MyRepository {
            private final Collection<Document> myMongoCollection;

            public Document findOne(int age) {
                return myMongoCollection.find(gt("age", age)).first();
            }
        }
        "#.to_string();

        let java = Java::new();
        let tree = java.full_parse(&code);
        let result = find_one(tree, &code).unwrap();

        assert_eq!(result.len(), 1);
        let FindOne { namespace: _, predicate} = &result[0] else {
            panic!()
        };

        assert_eq!(*predicate, GreaterThan {
            field: "age".to_string(),
            value: Reference("age".to_string(), "any".to_string())
        })
    }
}