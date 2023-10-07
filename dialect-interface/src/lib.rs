use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;

use tree_sitter::{InputEdit, Point, Tree};
use url::Url;

use crate::FileResourceChange::{Full, Range};

#[derive(Clone)]
struct FileResourceId {
    url: Url,
    version: u32
}

struct FileResourceChangeRangePosition {
    row: usize,
    column: usize
}

struct FileResourceChangeRange {
    start: FileResourceChangeRangePosition,
    end: FileResourceChangeRangePosition
}

enum FileResourceChange {
    Range(FileResourceChangeRange, String),
    Full(String)
}

#[derive(Clone)]
struct FileResource {
    id: FileResourceId,
    language: Rc<String>,
    source: String,
    tree: RefCell<Tree>,
    parser: Rc<dyn DialectParser>
}

trait DialectParser {
    fn full_parse(&self, contents: &String) -> RefCell<Tree>;
    fn reparse(&self, contents: &String, original: RefCell<Tree>) -> RefCell<Tree>;
}

fn resolve_byte_position(contents: &String, points: [&Point;3]) -> [usize;3] {
    let mut pos: [usize; 3] = [0, 0, 0];
    let mut row: usize = 0;
    let mut col: usize = 0;
    let mut byte: usize = 0;
    let mut match_count: usize = 0;

    for c in contents.chars() {
        byte += 1;

        match c {
            '\n' => {
                row += 1;
                col += 0;
            }
            _ => {
                col += 1;
            }
        }

        for x in 0..2 {
            if points[x].row == row && points[x].column == col {
                pos[x] = byte;
                match_count += 1;
            }
        }

        if match_count == 3 {
            break;
        }
    }

    return pos;
}

fn apply_changes_to_string(contents: &String, changes: &[FileResourceChange]) -> (Option<InputEdit>, String) {
    let mut result = contents.clone();
    let mut start_position = Point { row: usize::MAX, column: usize::MAX };
    let mut old_end_position = Point { row: usize::MIN, column: usize::MIN };
    let mut new_end_position = Point { row: usize::MIN, column: usize::MIN };
    let mut start_byte: usize = 0;
    let mut old_end_byte: usize = 0;
    let mut new_end_byte: usize = 0;

    for change in changes {
        if let Range(boundaries, new_text) = change {
            start_position.row = min(start_position.row, boundaries.start.row);
            start_position.column = min(start_position.column, boundaries.start.column);

            old_end_position.row = max(old_end_position.row, boundaries.end.row);
            old_end_position.column = max(old_end_position.column, boundaries.end.column);

            new_end_position.row = max(new_end_position.row, start_position.row + new_text.len());
            new_end_position.column = max(new_end_position.column, start_position.column + new_text.len());

            [start_byte, old_end_byte, new_end_byte] = resolve_byte_position(contents, [&start_position, &old_end_position, &new_end_position]);

            result.replace_range(start_byte ..= old_end_byte, new_text.as_str());
        } else if let Full(edit) = change {
            return (None, edit.clone());
        }
    }

    return (Some(InputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_position,
        old_end_position,
        new_end_position,
    }), result)
}

impl FileResource {
    pub fn new(url: Url, contents: &String, language_id: &Rc<String>, parser: &Rc<dyn DialectParser>) -> Box<FileResource> {
        let base_tree = parser.full_parse(contents);

        return Box::new(FileResource {
            id: FileResourceId {
                url,
                version: 1
            },
            language: Rc::clone(language_id),
            source: contents.to_owned(),
            tree: base_tree,
            parser: Rc::clone(parser)
        });
    }

    pub fn update(&mut self, changes: &[FileResourceChange]) -> Box<FileResource> {
        let (maybe_edit, new_source) = apply_changes_to_string(&self.source, changes);
        match maybe_edit {
            Some(edit) => {
                self.tree.borrow_mut().edit(&edit);
                self.tree = self.parser.reparse(&new_source, RefCell::clone(&self.tree));
                self.source = new_source;
            }
            None => {
                self.tree = self.parser.full_parse(&new_source);
                self.source = new_source;
            }
        }

        return Box::new(self.clone());
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
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
    fn can_parse_fully_a_file() {
        let java: Rc<dyn DialectParser> = Rc::new(Java::new());
        let file = FileResource::new(Url::parse("file://ws/test.java").unwrap(), &"class MyClass {}".to_string(), &Rc::new("java".to_string()), &Rc::clone(&java));

        assert_eq!("class MyClass {}", file.source);
    }

    #[test]
    fn can_do_full_edit_of_a_file() {
        let java: Rc<dyn DialectParser> = Rc::new(Java::new());
        let mut file = FileResource::new(Url::parse("file://ws/test.java").unwrap(), &"class MyClass {}".to_string(), &Rc::new("java".to_string()), &Rc::clone(&java));
        file.update(&[ Full("class Y {}".to_string()) ]);

        assert_eq!("class Y {}", file.source);
    }

    #[test]
    fn can_do_an_incrementa_edit_of_file() {
        let java: Rc<dyn DialectParser> = Rc::new(Java::new());
        let mut file = FileResource::new(Url::parse("file://ws/test.java").unwrap(), &"class MyClass {}".to_string(), &Rc::new("java".to_string()), &Rc::clone(&java));
        let start = FileResourceChangeRangePosition {
            column: 15,
            row: 0
        };

        let end = FileResourceChangeRangePosition {
            column: 14,
            row: 0
        };

        let change = Range(FileResourceChangeRange { start, end }, "private int X;".to_string());

        file.update(&[ change ]);

        assert_eq!("class MyClass {private int X;}", file.source);
    }
}
