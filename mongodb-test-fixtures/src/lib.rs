#![feature(custom_test_frameworks)]

use std::rc::Rc;

use mongodb::{IndexModel, Namespace};
use mongodb::bson::Document;
use mongodb::sync::Client;
use testcontainers::GenericImage;
use testcontainers::clients::Cli;
use testcontainers::core::WaitFor;

use crate::version::MongoDBVersion;

mod version;

pub struct MongoDBSandbox {
    testcontainers: Rc<Cli>,
    coll_documents: Vec<(Namespace, Document)>,
    coll_indexes: Vec<(Namespace, IndexModel)>,
    version: MongoDBVersion
}

impl MongoDBSandbox {
    fn new(version: MongoDBVersion) -> MongoDBSandbox {
        return MongoDBSandbox {
            testcontainers: Rc::new(Cli::default()),
            coll_documents: vec![],
            coll_indexes: vec![],
            version
        }
    }

    pub fn insert(&mut self, namespace: &'static str, docs: Vec<Document>) -> &Self {
        docs.iter().for_each(|d| {
            self.coll_documents.push((namespace.to_string().parse().unwrap(), d.clone()));
        });

        return self;
    }

    pub fn create_index(&mut self, namespace: &'static str, docs: Vec<IndexModel>) -> &Self {
        docs.iter().for_each(|d| {
            self.coll_indexes.push((namespace.to_string().parse().unwrap(), d.clone()));
        });

        return self;
    }

    pub fn run(&self, test: fn(Client)){
        let container = self.testcontainers.run(
            GenericImage::new("docker.io/mongo", self.version.tag())
                .with_wait_for(WaitFor::message_on_stdout("Waiting for connections"))
                .with_env_var("MONGO_INITDB_DATABASE", "test")
                .with_env_var("MONGO_INITDB_ROOT_USERNAME", "root")
                .with_env_var("MONGO_INITDB_ROOT_PASSWORD", "root")
        );

        let host_port = container.get_host_port_ipv4(27017);
        let url = format!("mongodb://root:root@127.0.0.1:{host_port}/");
        let client = Client::with_uri_str(&url).unwrap();

        for (ns, index) in &self.coll_indexes {
            client.database(&*ns.db).collection::<Document>(&*ns.coll)
                .create_index(index.clone(), None).expect(
                &*format!("[{:?}] Could not create index: {:?}", ns, index)
            );
        }

        for (ns, document) in &self.coll_documents {
            client.database(&*ns.db).collection::<Document>(&*ns.coll)
                .insert_one(document, None).expect(
                &*format!("[{:?}] Could not insert document: {:?}", ns, document)
            );
        }

        test(client);
    }
}

#[cfg(test)]
mod test {
    use mongodb::bson::{doc, Document};
    use mongodb::IndexModel;
    use mongodb::sync::Client;
    use test_case::test_case;
    use crate::MongoDBSandbox;

    use crate::version::MongoDBVersion;

    #[test_case(MongoDBVersion::V7 ; "version is 7")]
    #[test_case(MongoDBVersion::V6 ; "version is 6")]
    #[test_case(MongoDBVersion::V5 ; "version is 5")]
    fn creates_indexes_on_any_version(version: MongoDBVersion) {
        MongoDBSandbox::new(version)
            .create_index("test.test", vec! [
                IndexModel::builder().keys(doc! { "field": 1 } ).build()
            ]).run(|client: Client| {
            let index_names = client.database("test").collection::<Document>("test")
                .list_index_names().unwrap();

            assert_eq!(index_names.len(), 2);
            assert_eq!(index_names[0], "_id_");
            assert_eq!(index_names[1], "field_1");
        });
    }

    #[test_case(MongoDBVersion::V7 ; "version is 7")]
    #[test_case(MongoDBVersion::V6 ; "version is 6")]
    #[test_case(MongoDBVersion::V5 ; "version is 5")]
    fn inserts_documents_on_any_version(version: MongoDBVersion) {
        MongoDBSandbox::new(version)
            .insert("test.test", vec! [ doc! {
                "test": 1
            }])
            .run(|client: Client| {
            let Some(result) = client.database("test").collection::<Document>("test")
                .find_one(Some(doc!{}), None).unwrap() else {
                panic!("Expecting one document.")
            };

            assert_eq!(result.get_i32("test"), Ok(1));
        });
    }
}