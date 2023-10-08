use crate::filter::FilterOperator;

pub struct ExecutionNamespace {
    database: String,
    collection: String
}
pub enum Execution {
    FindOne { namespace: ExecutionNamespace, predicate: FilterOperator },
    FindMany { namespace: ExecutionNamespace, predicate: FilterOperator },
}