use std::error::Error;

use mongodb::bson::Document;
use mongodb::Namespace;

#[derive(Eq, PartialEq, Debug)]
pub enum SchemaRegularIndexPredicate {
    Ascending(String),
    Descending(String),
    Text(String),
    Unknown(String, String),
}

#[derive(Eq, PartialEq, Debug)]
pub struct SchemaRegularIndex {
    pub name: String,
    pub predicates: Vec<SchemaRegularIndexPredicate>,
}

pub struct Schema {
    pub regular_indexes: Vec<SchemaRegularIndex>,
    pub samples: Vec<Document>,
    pub normalized: Document,
}

pub trait InferSchema {
    fn infer_schema(&self, namespace: &Namespace) -> Result<Schema, Box<dyn Error + Send + Sync>>;
}
