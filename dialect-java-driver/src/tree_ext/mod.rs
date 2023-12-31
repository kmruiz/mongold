use tree_sitter::Node;

pub mod friendly_capture;
pub mod infer_mongodb_namespace;
pub mod predicate_from_driver_method;

pub fn optional_node_to_string(node: &Option<Node>, code: &String) -> String {
    return node
        .map(|x| {
            x.utf8_text(code.as_bytes())
                .unwrap()
                .to_string()
                .replace("\"", "")
        })
        .unwrap_or("<unknown>".to_string());
}
