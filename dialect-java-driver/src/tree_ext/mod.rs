use tree_sitter::Node;

pub mod infer_mongodb_namespace;

pub fn optional_node_to_string(node: &Option<Node>, code: &String) -> String {
    return node.map(|x| x.utf8_text(code.as_bytes()).unwrap().to_string().replace("\"", "")).unwrap_or("<unknown>".to_string());
}

pub fn node_to_string(node: Node, code: &String) -> String {
    return node.utf8_text(code.as_bytes()).unwrap().replace("\"", "");
}