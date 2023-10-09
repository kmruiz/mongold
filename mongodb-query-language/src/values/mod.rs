#[derive(PartialEq, Debug)]
pub enum Value {
    String(String),
    Integer(i32),
    Floating(f32),
    Decimal128(f64),
    Date(i64),
    Object(Vec<(String, Value)>),
    Array(Vec<Value>),
    ObjectId(String),
    Reference(String, String),
}
