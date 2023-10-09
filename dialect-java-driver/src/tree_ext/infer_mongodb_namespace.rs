use crate::tree_ext::optional_node_to_string;
use mongodb_query_language::execution::ExecutionNamespace;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use tree_sitter::Node;

const COLLECTION_WITH_NAMESPACE: &str = include_str!("queries/collection_with_namespace.scm");

pub fn infer_mongodb_namespace(
    root: Node,
    code: &String,
) -> Result<HashMap<String, ExecutionNamespace>, Box<dyn Error + Sync + Send>> {
    let mut result: HashMap<String, ExecutionNamespace> = HashMap::new();
    let collections_query =
        tree_sitter::Query::new(tree_sitter_java::language(), COLLECTION_WITH_NAMESPACE)?;
    let mut cursor = tree_sitter::QueryCursor::new();

    let all_matches = cursor.matches(&collections_query, root, code.as_bytes());
    let javadoc_idx = collections_query.capture_index_for_name("javadoc").unwrap();
    let field_name_idx = collections_query.capture_index_for_name("field").unwrap();
    let extract_namespace_regex = Regex::new(r"@mongodb\.namespace (\w+).(\w+)")?;

    for each_match in all_matches {
        let mut javadoc_node: Option<Node> = None;
        let mut field_name_node: Option<Node> = None;

        for capture in each_match.captures {
            if capture.index == javadoc_idx {
                javadoc_node = Some(capture.node);
            } else if capture.index == field_name_idx {
                field_name_node = Some(capture.node);
            }
        }

        let javadoc = optional_node_to_string(&javadoc_node, code);
        let captures = extract_namespace_regex.captures(javadoc.as_str());

        if let Some(capture) = captures {
            if capture.len() == 3 {
                let field_name = optional_node_to_string(&field_name_node, code);
                result.insert(
                    field_name.clone(),
                    ExecutionNamespace {
                        database: capture.get(1).map(|x| x.as_str().to_string()),
                        collection: capture.get(2).map(|x| x.as_str().to_string()),
                        reference_name: field_name,
                    },
                );
            }
        }
    }

    return Ok(result);
}
