use dialect_interface::DialectParser;
use dialect_java_driver::Java;
use std::collections::HashMap;
use std::rc::Rc;

pub struct LanguageBasedDialectResolver {
    resolvers: HashMap<&'static str, Rc<dyn DialectParser>>,
}

pub trait DialectResolver {
    fn resolve_dialect(
        &self,
        language_id: &String,
        contents: &String,
    ) -> Option<Rc<dyn DialectParser>>;
}

impl LanguageBasedDialectResolver {
    pub fn new() -> Rc<dyn DialectResolver> {
        let mut resolvers = HashMap::new();
        resolvers.insert("java", Java::new());

        return Rc::new(LanguageBasedDialectResolver { resolvers });
    }
}

impl DialectResolver for LanguageBasedDialectResolver {
    fn resolve_dialect(
        &self,
        language_id: &String,
        _contents: &String,
    ) -> Option<Rc<dyn DialectParser>> {
        return match self.resolvers.get(language_id.as_str()) {
            Some(parser) => Some(Rc::clone(parser)),
            None => None,
        };
    }
}
