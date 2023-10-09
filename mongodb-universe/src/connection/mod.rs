use std::collections::HashSet;
use std::error::Error;
use std::fmt::format;

use mongodb::{IndexModel, Namespace};
use mongodb::bson::{Bson, doc, Document};
use mongodb::bson::Bson::Array;
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

fn merge_document(left: &Document, right: &Document) -> Document {
    let mut parsed_keys: HashSet<&String> = HashSet::new();
    let mut result = doc! {};

    for key in left.keys() {
        let lval = left.get(key).unwrap_or(&Bson::Null);
        let rval = right.get(key).unwrap_or(&Bson::Null);

        if lval.as_document() != None && rval.as_document() != None {
            result.insert(key, merge_document(lval.as_document().unwrap(), rval.as_document().unwrap()));
        } else if lval != rval {
            result.insert(key, vec! [ lval, rval ]);
        } else {
            result.insert(key, vec! [ lval ]);
        }

        parsed_keys.insert(key);
    }

    for key in right.keys() {
        if parsed_keys.contains(key) {
            continue;
        }

        let lval = left.get(key).unwrap_or(&Bson::Null);
        let rval = right.get(key).unwrap_or(&Bson::Null);

        if lval.as_document() != None && rval.as_document() != None {
            result.insert(key, merge_document(lval.as_document().unwrap(), rval.as_document().unwrap()));
        } else if lval != rval {
            result.insert(key, vec! [ lval, rval ]);
        } else {
            result.insert(key, vec! [ lval ]);
        }

        parsed_keys.insert(key);
    }

    return result;
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
            "$sample": { "size": 5 },
        }, doc! {
            "$sort": { "_id": 1 },
        }], None)?.map(|r| { r.unwrap() }).collect();

        let mut normalized = doc![];
        if samples.len() > 0 {
            normalized = samples[0].clone();
            for sample in &samples[1..] {
                normalized = merge_document(&normalized, &sample);
            }
        }

        return Ok(Schema {
            regular_indexes,
            samples,
            normalized
        })
    }
}

#[cfg(test)]
mod tests {
    use mongodb::{IndexModel, Namespace};
    use mongodb::bson::{Bson, doc};
    use mongodb::sync::Client;
    use test_case::test_case;

    use mongodb_test_fixtures::MongoDBSandbox;
    use mongodb_test_fixtures::version::MongoDBVersion;

    use crate::schema::{InferSchema, SchemaRegularIndex};
    use crate::schema::SchemaRegularIndexPredicate::Ascending;

    #[test_case(MongoDBVersion::V7 ; "version is 7")]
    #[test_case(MongoDBVersion::V6 ; "version is 6")]
    #[test_case(MongoDBVersion::V5 ; "version is 5")]
    fn resolves_schemas_on_any_supported_version_replicaset(version: MongoDBVersion) {
        MongoDBSandbox::new(version)
            .insert("test.withIndexes", vec![
                doc! {
                    "indexed": true
                }, doc! {
                    "indexed": true,
                    "not_indexed": false
                }
            ])
            .create_index("test.withIndexes", vec! [
                IndexModel::builder().keys(doc! { "indexed": 1 } ).build()
            ]).run(|client: Client| {
            let ns = Namespace::new("test", "withIndexes");
            let schema = client.infer_schema(&ns).unwrap();

            assert_eq!(schema.regular_indexes.len(), 2);
            let id_index = &schema.regular_indexes[0];
            let indexed_index = &schema.regular_indexes[1];

            assert_eq!(id_index, &SchemaRegularIndex { name: "_id_".to_string(), predicates: vec![ Ascending("_id".to_string()) ]});
            assert_eq!(indexed_index, &SchemaRegularIndex { name: "indexed_1".to_string(), predicates: vec![ Ascending("indexed".to_string()) ]});

            assert_eq!(schema.samples.len(), 2);
            assert_eq!(schema.samples[0].get_bool("indexed").unwrap(), true);
            assert_eq!(schema.samples[1].get_bool("not_indexed").unwrap(), false);

            assert_eq!(schema.normalized.get_array("indexed").unwrap()[0].as_bool(), Some(true));
            assert_eq!(schema.normalized.get_array("indexed").unwrap().len(), 1);

            assert_eq!(schema.normalized.get_array("not_indexed").unwrap()[0], Bson::Null);
            assert_eq!(schema.normalized.get_array("not_indexed").unwrap()[1].as_bool(), Some(false));
        });
    }
}