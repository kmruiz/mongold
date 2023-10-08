use std::error::Error;
use std::rc::Rc;
use tree_sitter::Tree;
use mongodb_query_language::execution::Execution;

pub fn find_one(_tree: Rc<Tree>) -> Result<Vec<Execution>, Box<dyn Error + Sync + Send>> {
    return Ok(vec![]);
}