use mongodb_query_language::filter::FilterOperator;
use mongodb_query_language::filter::FilterOperator::{Equals, GreaterThan};
use mongodb_query_language::values::Value;
use mongodb_query_language::values::Value::Reference;
use crate::tree_ext::optional_node_to_string;

pub fn predicate_from_driver_method(operator: &String, field: String, value: Value) -> FilterOperator {
    return match operator.as_str() {
        "eq" => Equals {
            field,
            value
        },
        "gt" => GreaterThan {
            field,
            value
        },
        _ => {
            Equals {
                field,
                value
            }
        }
    }
}