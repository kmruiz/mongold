use std::cell::RefCell;
use std::error::Error;
use tree_sitter::Tree;
use crate::filter::FilterOperator;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct ExecutionNamespace {
    pub database: Option<String>,
    pub collection: Option<String>,
    pub reference_name: String
}

impl ExecutionNamespace {
    pub fn empty(reference: String) -> ExecutionNamespace {
        return ExecutionNamespace {
            database: None,
            collection: None,
            reference_name: reference
        }
    }
}
#[derive(PartialEq, Debug)]
pub enum Execution {
    FindOne { namespace: ExecutionNamespace, predicate: FilterOperator },
    FindMany { namespace: ExecutionNamespace, predicate: FilterOperator },
}

pub trait ExecutionProcessor {
    fn process(tree: RefCell<Tree>, code: &String) -> Result<Vec<Execution>, Box<dyn Error + Sync + Send>>;
}