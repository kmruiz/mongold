use std::error::Error;
use std::rc::Rc;
use mongodb::{IndexModel, Namespace};
use mongodb::bson::{doc, Document};
use mongodb::sync::{Client, Collection};
use crate::schema::{InferSchema, Schema, SchemaRegularIndex};
use crate::schema::SchemaRegularIndexPredicate::{Ascending, Descending, Text, Unknown};

pub fn connect(url: String) -> Result<Client, Box<dyn Error + Sync + Send>> {
    let client = Client::with_uri_str(url)?;
    return Ok(client);
}

fn map_regular_index(model: IndexModel) -> SchemaRegularIndex {
    let predicates = model.keys.iter().map(|(key, value)| {
        match value.as_i32() {
            Some(direction) => return match direction {
                -1 => Descending(key.clone()),
                1 => Ascending(key.clone()),
                _ => Unknown(key.clone(), format!("{}", direction))
            },
            None => {}
        }

        match value.as_str() {
            Some("text") => return Text(key.clone()),
            _ => {}
        }

        return Unknown(key.clone(), value.to_string());
    });

    return SchemaRegularIndex {
        name: model.options.unwrap().name.unwrap(),
        predicates: predicates.collect()
    };
}

impl InferSchema for Client {
    fn infer_schema(&self, namespace: &Namespace) -> Result<Schema, Box<dyn Error + Send + Sync>> {
        let db = self.database(namespace.db.as_str());
        let coll: Collection<Document> = db.collection(namespace.coll.as_str());

        let regular_indexes_from_mongodb = coll.list_indexes(None)?;
        let regular_indexes: Vec<SchemaRegularIndex> = regular_indexes_from_mongodb
            .map(|r| { r.unwrap() })
            .map(map_regular_index)
            .collect();

        let samples: Vec<Document> = coll.aggregate([doc! {
            "$sample": { "number": 5 }
        }], None)?.map(|r| { r.unwrap() }).collect();

        return Ok(Schema {
            regular_indexes,
            samples
        })
    }
}

#[cfg(test)]
mod tests {

}