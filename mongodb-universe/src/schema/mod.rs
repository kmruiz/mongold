use std::error::Error;
use mongodb::bson::Document;
use mongodb::Namespace;
use mongodb_query_language::values::Value;

pub enum SchemaRegularIndexPredicate {
    Ascending(String),
    Descending(String),
    Text(String),
    Unknown(String, String),
}

pub struct SchemaRegularIndex {
    pub name: String,
    pub predicates: Vec<SchemaRegularIndexPredicate>,
}

pub struct Schema {
    pub regular_indexes: Vec<SchemaRegularIndex>,
    pub samples: Vec<Document>
}

pub trait InferSchema {
    fn infer_schema(&self, namespace: &Namespace) -> Result<Schema, Box<dyn Error + Send + Sync>>;
}