use crate::version::MongoDBVersion::V7;

#[derive(Clone, Copy)]
pub enum MongoDBVersion {
    V7,
    V6,
    V5,
}

impl MongoDBVersion {
    pub fn tag(&self) -> &'static str {
        return match self {
            MongoDBVersion::V7 => "7.0.2",
            MongoDBVersion::V6 => "6.0.10",
            MongoDBVersion::V5 => "5.0.21",
        };
    }

    pub fn default() -> MongoDBVersion {
        return V7;
    }
}

impl ToString for MongoDBVersion {
    fn to_string(&self) -> String {
        return self.tag().to_string();
    }
}
