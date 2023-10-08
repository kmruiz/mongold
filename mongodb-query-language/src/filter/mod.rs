use crate::values::Value;

pub enum FilterOperator {
    And { predicates: Vec<FilterOperator> },
    Or { predicates: Vec<FilterOperator> },
    Not { predicates: Vec<FilterOperator> },
    Equals { field: String, value: Value }
}
