use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use lsp_types::{DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams};
use tracing::info;
use tree_sitter::Tree;
use url::Url;

use dialect_interface::{DialectParser, FileResource, FileResourceChange, FileResourceChangeRange};
use crate::dialect_resolver::DialectResolver;

pub struct Workspace {
    open_files: HashMap<Url, RefCell<FileResource>>,
}

impl Workspace {
    pub fn new() -> RefCell<Workspace> {
        return RefCell::new(
            Workspace { open_files: HashMap::new() }
        )
    }

    pub fn open(&mut self, params: &DidOpenTextDocumentParams, resolver: Rc<dyn DialectResolver>) -> Option<RefCell<Tree>> {
        let url = &params.text_document.uri;
        return match resolver.resolve_dialect(&params.text_document.language_id, &params.text_document.text) {
            Some(dialect) => {
                let resource = FileResource::new(url.clone(), &params.text_document.text, Rc::new(params.text_document.language_id.clone()), Rc::clone(&dialect));
                self.open_files.insert(url.clone(), resource.clone());
                return Some(resource.borrow().tree());
            }
            None => {
                let last_segment = url.path_segments().map(|s| { s.last() }).unwrap_or_else(|| None);
                info!(file_name = last_segment, language_id = &params.text_document.language_id, "Unknown language requested.");
                None
            }
        }
    }

    pub fn update(&mut self, params: &DidChangeTextDocumentParams) -> Option<RefCell<Tree>> {
        let Some(file_resource) = self.open_files.get(&params.text_document.uri) else {
            return None;
        };

        let changes = params.content_changes.iter().map(|change| {
            match change.range {
                Some(range) => {
                    FileResourceChange::Range(
                        FileResourceChangeRange::new(
                            range.start.line as usize,
                            range.start.character as usize,
                            range.end.line as usize,
                            range.end.character as usize
                        ),
                        change.text.clone()
                    )
                }
                None => {
                    FileResourceChange::Full(change.text.clone())
                }
            }
        }).collect::<Vec<FileResourceChange>>();

        file_resource.borrow_mut().update(&changes);
        return Some(file_resource.borrow().tree());
    }

    pub fn close(&mut self, params: &DidCloseTextDocumentParams) {
        self.open_files.remove(&params.text_document.uri);
    }
}


#[cfg(test)]
mod tests {
    use lsp_types::{TextDocumentContentChangeEvent, TextDocumentItem, VersionedTextDocumentIdentifier};
    use tree_sitter::Parser;

    use super::*;

    struct Java {
        parser: RefCell<Parser>
    }

    impl Java {
        fn new() -> Self {
            let mut parser = Parser::new();
            parser.set_language(tree_sitter_java::language()).expect("Error loading Java grammar.");

            return Java {
                parser: RefCell::new(parser)
            }
        }
    }

    impl DialectResolver for Java {
        fn resolve_dialect(&self, language_id: &String, _contents: &String) -> Option<Rc<dyn DialectParser>> {
            return Some(Rc::new(Java::new()));
        }
    }

    impl DialectParser for Java {
        fn full_parse(&self, contents: &String) -> RefCell<Tree> {
            let Some(tree) = self.parser.borrow_mut().parse(contents, None) else {
                panic!("At full parse");
            };

            return RefCell::new(tree);
        }

        fn reparse(&self, contents: &String, original: RefCell<Tree>) -> RefCell<Tree> {
            let Some(tree) = self.parser.borrow_mut().parse(contents, Some(&*original.borrow())) else {
                panic!("At reparse");
            };

            return RefCell::new(tree);
        }
    }

    #[test]
    fn can_open_a_new_file() {
        let java = Rc::new(Java::new());
        let ws = Workspace::new();
        let tree = ws.borrow_mut().open(&DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::parse("file://my-ws/test.java").unwrap(),
                language_id: "java".to_string(),
                version: 0,
                text: "class X {}".to_string()
            }
        }, java);

        assert_eq!(tree.is_some(), true);
    }

    #[test]
    fn can_edit_an_existing_file() {
        let java = Rc::new(Java::new());
        let ws = Workspace::new();
        ws.borrow_mut().open(&DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::parse("file://my-ws/test.java").unwrap(),
                language_id: "java".to_string(),
                version: 0,
                text: "class X {}".to_string()
            }
        }, java);

        let tree = ws.borrow_mut().update(&DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: Url::parse("file://my-ws/test.java").unwrap(),
                version: 0,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent {
                    text: "class Y {}".to_string(),
                    range: None,
                    range_length: None
                }
            ]
        });

        assert_eq!(tree.is_some(), true);
    }
}
