use crate::values::Value;

#[derive(PartialEq, Debug)]
pub enum FilterOperator {
    And { predicates: Vec<FilterOperator> },
    Or { predicates: Vec<FilterOperator> },
    Not { predicates: Vec<FilterOperator> },
    Equals { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    LessThan { field: String, value: Value },
    GreaterThanOrEquals { field: String, value: Value },
    LessThanOrEquals { field: String, value: Value },
}
