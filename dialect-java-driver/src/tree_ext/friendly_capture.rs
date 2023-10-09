use tree_sitter::{Node, QueryMatch};

pub trait FriendlyCapture {
    fn capture(&self, capture_indexes: Vec<u32>) -> Vec<Option<Node>>;
}

impl FriendlyCapture for QueryMatch<'_, '_> {
    fn capture(&self, capture_indexes: Vec<u32>) -> Vec<Option<Node>> {
        let mut result: Vec<Option<Node>> = Vec::with_capacity(capture_indexes.len());
        result.resize(capture_indexes.len(), None);

        for capture in self.captures {
            let mut idx = 0;
            for index in &capture_indexes {
                if *index == capture.index {
                    result[idx] = Some(capture.node)
                }
                idx += 1;
            }
        }

        return result;
    }

}