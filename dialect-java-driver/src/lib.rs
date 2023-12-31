use crate::use_cases::find_one::find_one;
use dialect_interface::DialectParser;
use mongodb_query_language::execution::{Execution, ExecutionProcessor};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use tree_sitter::{Parser, Tree};

mod tree_ext;
mod use_cases;

pub struct Java {
    parser: RefCell<Parser>,
}

impl Java {
    pub fn new() -> Rc<dyn DialectParser> {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_java::language())
            .expect("Error loading Java grammar.");

        return Rc::new(Java {
            parser: RefCell::new(parser),
        });
    }
}

impl ExecutionProcessor for Java {
    fn process(
        tree: RefCell<Tree>,
        code: &String,
    ) -> Result<Vec<Execution>, Box<dyn Error + Sync + Send>> {
        let mut result = vec![];
        result.append(&mut find_one(tree, code)?);

        return Ok(result);
    }
}

impl DialectParser for Java {
    fn full_parse(&self, contents: &String) -> RefCell<Tree> {
        let Some(tree) = self.parser.borrow_mut().parse(contents, None) else {
            todo!("full_parse error handling.")
        };

        return RefCell::new(tree);
    }

    fn reparse(&self, contents: &String, original: RefCell<Tree>) -> RefCell<Tree> {
        let Some(tree) = self
            .parser
            .borrow_mut()
            .parse(contents, Some(&*original.borrow()))
        else {
            todo!("reparse error handling.")
        };

        return RefCell::new(tree);
    }
}
