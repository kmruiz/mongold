use mongodb_query_language::values::Value;

pub struct SchemaField {
    name: String,
    values: Vec<Value>
}

pub enum SchemaRegularIndexPredicate {
    Ascending(String),
    Descending(String),
    Text(String),
    Wildcard(String)
}

pub struct SchemaRegularIndex {
    name: String,
    predicates: Vec<SchemaRegularIndexPredicate>,

}