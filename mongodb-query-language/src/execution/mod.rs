use std::error::Error;
use std::rc::Rc;
use tree_sitter::Tree;
use crate::filter::FilterOperator;

pub struct ExecutionNamespace {
    database: String,
    collection: String
}
pub enum Execution {
    FindOne { namespace: ExecutionNamespace, predicate: FilterOperator },
    FindMany { namespace: ExecutionNamespace, predicate: FilterOperator },
}

pub trait ExecutionProcessor {
    fn process(tree: Rc<Tree>) -> Result<Vec<Execution>, Box<dyn Error + Sync + Send>>;
}