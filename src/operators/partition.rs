use crate::ast::Selection;
use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::{Array, Level, Value};

use super::select::selection_indices;

pub struct Partition {
    selection: Selection,
}

impl Partition {
    pub fn new(selection: Selection) -> Self {
        Self { selection }
    }
}

impl Transform for Partition {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(arr) => {
                let len = arr.len() as i64;
                let mut split_points = selection_indices(&self.selection, len);
                split_points.sort();
                split_points.dedup();

                let split_points: Vec<usize> = split_points
                    .into_iter()
                    .filter(|&i| i > 0 && i < arr.len())
                    .collect();

                if split_points.is_empty() {
                    return Ok(Value::Array(Array::from((
                        vec![Value::Array(arr)],
                        Level::Line,
                    ))));
                }

                let mut result: Vec<Value> = Vec::new();
                let mut start = 0;
                for split_at in split_points {
                    let chunk: Vec<Value> = arr.elements[start..split_at]
                        .iter()
                        .map(|v| v.deep_copy())
                        .collect();
                    result.push(Value::Array(Array::from((chunk, arr.level))));
                    start = split_at;
                }
                let chunk: Vec<Value> = arr.elements[start..]
                    .iter()
                    .map(|v| v.deep_copy())
                    .collect();
                result.push(Value::Array(Array::from((chunk, arr.level))));

                Ok(Value::Array(Array::from((result, Level::Line))))
            }
            Value::Text(s) => {
                let chars: Vec<char> = s.chars().collect();
                let len = chars.len() as i64;
                let mut split_points = selection_indices(&self.selection, len);
                split_points.sort();
                split_points.dedup();

                let split_points: Vec<usize> = split_points
                    .into_iter()
                    .filter(|&i| i > 0 && i < chars.len())
                    .collect();

                if split_points.is_empty() {
                    return Ok(Value::Array(Array::from((
                        vec![Value::Text(s)],
                        Level::Line,
                    ))));
                }

                let mut result: Vec<Value> = Vec::new();
                let mut start = 0;
                for split_at in split_points {
                    let chunk: String = chars[start..split_at].iter().collect();
                    result.push(Value::Text(chunk));
                    start = split_at;
                }
                let chunk: String = chars[start..].iter().collect();
                result.push(Value::Text(chunk));

                Ok(Value::Array(Array::from((result, Level::Word))))
            }
            Value::Number(n) => Ok(Value::Number(n)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{SelectItem, Slice};

    fn text(s: &str) -> Value {
        Value::Text(s.to_string())
    }

    fn line_array(lines: &[&str]) -> Value {
        Value::Array(Array::from((
            lines.iter().map(|s| text(s)).collect(),
            Level::Line,
        )))
    }

    #[test]
    fn partition_array_single_index() {
        let input = line_array(&["a", "b", "c", "d", "e"]);
        let sel = Selection {
            items: vec![SelectItem::Index(2)],
        };
        let result = Partition::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                match &arr.elements[0] {
                    Value::Array(chunk) => {
                        assert_eq!(chunk.len(), 2);
                        assert_eq!(chunk.elements[0], text("a"));
                        assert_eq!(chunk.elements[1], text("b"));
                    }
                    _ => panic!("expected array"),
                }
                match &arr.elements[1] {
                    Value::Array(chunk) => {
                        assert_eq!(chunk.len(), 3);
                        assert_eq!(chunk.elements[0], text("c"));
                    }
                    _ => panic!("expected array"),
                }
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn partition_array_multiple_indices() {
        let input = line_array(&["a", "b", "c", "d", "e"]);
        let sel = Selection {
            items: vec![SelectItem::Index(1), SelectItem::Index(3)],
        };
        let result = Partition::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                match &arr.elements[0] {
                    Value::Array(chunk) => assert_eq!(chunk.len(), 1),
                    _ => panic!("expected array"),
                }
                match &arr.elements[1] {
                    Value::Array(chunk) => assert_eq!(chunk.len(), 2),
                    _ => panic!("expected array"),
                }
                match &arr.elements[2] {
                    Value::Array(chunk) => assert_eq!(chunk.len(), 2),
                    _ => panic!("expected array"),
                }
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn partition_array_chunking() {
        let input = line_array(&["a", "b", "c", "d", "e", "f"]);
        let sel = Selection {
            items: vec![SelectItem::Slice(Slice {
                start: None,
                end: None,
                step: Some(2),
            })],
        };
        let result = Partition::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                match &arr.elements[0] {
                    Value::Array(chunk) => {
                        assert_eq!(chunk.len(), 2);
                        assert_eq!(chunk.elements[0], text("a"));
                        assert_eq!(chunk.elements[1], text("b"));
                    }
                    _ => panic!("expected array"),
                }
                match &arr.elements[1] {
                    Value::Array(chunk) => {
                        assert_eq!(chunk.len(), 2);
                        assert_eq!(chunk.elements[0], text("c"));
                        assert_eq!(chunk.elements[1], text("d"));
                    }
                    _ => panic!("expected array"),
                }
                match &arr.elements[2] {
                    Value::Array(chunk) => {
                        assert_eq!(chunk.len(), 2);
                        assert_eq!(chunk.elements[0], text("e"));
                        assert_eq!(chunk.elements[1], text("f"));
                    }
                    _ => panic!("expected array"),
                }
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn partition_array_no_split() {
        let input = line_array(&["a", "b", "c"]);
        let sel = Selection {
            items: vec![SelectItem::Index(0)],
        };
        let result = Partition::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 1);
                match &arr.elements[0] {
                    Value::Array(chunk) => assert_eq!(chunk.len(), 3),
                    _ => panic!("expected array"),
                }
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn partition_string_single_index() {
        let input = text("hello");
        let sel = Selection {
            items: vec![SelectItem::Index(2)],
        };
        let result = Partition::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("he"));
                assert_eq!(arr.elements[1], text("llo"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn partition_string_multiple_indices() {
        let input = text("hello");
        let sel = Selection {
            items: vec![SelectItem::Index(1), SelectItem::Index(3)],
        };
        let result = Partition::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], text("h"));
                assert_eq!(arr.elements[1], text("el"));
                assert_eq!(arr.elements[2], text("lo"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn partition_string_chunking() {
        let input = text("abcdef");
        let sel = Selection {
            items: vec![SelectItem::Slice(Slice {
                start: None,
                end: None,
                step: Some(2),
            })],
        };
        let result = Partition::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr.elements[0], text("ab"));
                assert_eq!(arr.elements[1], text("cd"));
                assert_eq!(arr.elements[2], text("ef"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn partition_string_no_split() {
        let input = text("hello");
        let sel = Selection {
            items: vec![SelectItem::Index(0)],
        };
        let result = Partition::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr.elements[0], text("hello"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn partition_number_is_identity() {
        let input = Value::Number(42.0);
        let sel = Selection {
            items: vec![SelectItem::Index(2)],
        };
        let result = Partition::new(sel).apply(input).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn partition_negative_index() {
        let input = line_array(&["a", "b", "c", "d", "e"]);
        let sel = Selection {
            items: vec![SelectItem::Index(-2)],
        };
        let result = Partition::new(sel).apply(input).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                match &arr.elements[0] {
                    Value::Array(chunk) => assert_eq!(chunk.len(), 3),
                    _ => panic!("expected array"),
                }
                match &arr.elements[1] {
                    Value::Array(chunk) => assert_eq!(chunk.len(), 2),
                    _ => panic!("expected array"),
                }
            }
            _ => panic!("expected array"),
        }
    }
}
