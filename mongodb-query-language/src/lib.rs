use std::error::Error;
use std::rc::Rc;
use tree_sitter::Tree;
use crate::execution::Execution;

pub mod filter;
pub mod values;
pub mod execution;

trait ExecutionProcessor {
    fn process(tree: Rc<Tree>) -> Result<Vec<Execution>, Box<dyn Error + Sync + Send>>;
}