use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::{Array, Level, Value};

pub struct Columnate;

struct Cell {
    text: String,
    width: usize,
}

fn value_into_string(v: Value) -> String {
    match v {
        Value::Text(s) => s,
        Value::Number(n) => n.to_string(),
        Value::Array(arr) => arr.to_string(),
    }
}

impl Transform for Columnate {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(arr) => {
                // Flatten one level if first element is an array of arrays
                let is_nested = arr.elements.first().is_some_and(|v| {
                    if let Value::Array(inner) = v {
                        inner
                            .elements
                            .first()
                            .is_some_and(|v| matches!(v, Value::Array(_)))
                    } else {
                        false
                    }
                });
                let (elements, level) = if is_nested {
                    let mut flattened: Vec<Value> = Vec::new();
                    let mut inner_level = arr.level;
                    for elem in arr.elements {
                        if let Value::Array(inner) = elem {
                            inner_level = inner.level;
                            flattened.extend(inner.elements);
                        }
                    }
                    (flattened, inner_level)
                } else {
                    (arr.elements, arr.level)
                };

                // Convert to cells, taking ownership to avoid cloning Text values
                let rows: Vec<Vec<Cell>> = elements
                    .into_iter()
                    .map(|row| match row {
                        Value::Array(inner) => inner
                            .elements
                            .into_iter()
                            .map(|v| {
                                let text = value_into_string(v);
                                let width = text.chars().count();
                                Cell { text, width }
                            })
                            .collect(),
                        other => {
                            let text = value_into_string(other);
                            let width = text.chars().count();
                            vec![Cell { text, width }]
                        }
                    })
                    .collect();

                if rows.is_empty() {
                    return Ok(Value::Array(Array::from((vec![], level))));
                }

                let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
                let mut col_widths = vec![0usize; max_cols];
                for row in &rows {
                    for (i, cell) in row.iter().enumerate() {
                        col_widths[i] = col_widths[i].max(cell.width);
                    }
                }

                let result_elements: Vec<Value> = rows
                    .into_iter()
                    .map(|row| {
                        let last_idx = row.len().saturating_sub(1);
                        let cells: Vec<Value> = row
                            .into_iter()
                            .enumerate()
                            .map(|(i, cell)| {
                                if i == last_idx {
                                    Value::Text(cell.text)
                                } else {
                                    let target_width = col_widths.get(i).copied().unwrap_or(0);
                                    let padding = target_width.saturating_sub(cell.width);
                                    if padding == 0 {
                                        Value::Text(cell.text)
                                    } else {
                                        let mut padded = cell.text;
                                        padded.reserve(padding);
                                        for _ in 0..padding {
                                            padded.push(' ');
                                        }
                                        Value::Text(padded)
                                    }
                                }
                            })
                            .collect();
                        Value::Array(Array::from((cells, Level::Word)))
                    })
                    .collect();

                Ok(Value::Array(Array::from((result_elements, level))))
            }
            other => Ok(other),
        }
    }

    fn requires_full_input(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Value {
        Value::Text(s.to_string())
    }

    fn row(cells: Vec<&str>) -> Value {
        Value::Array(Array::from((
            cells.into_iter().map(text).collect(),
            Level::Word,
        )))
    }

    #[test]
    fn columnate_basic() {
        let input = Value::Array(Array::from((
            vec![
                Value::Array(Array::from((vec![text("name"), text("age")], Level::Word))),
                Value::Array(Array::from((vec![text("alice"), text("30")], Level::Word))),
                Value::Array(Array::from((vec![text("bob"), text("25")], Level::Word))),
            ],
            Level::Line,
        )));
        let result = Columnate.apply(input).unwrap();
        let expected = Value::Array(Array::from((
            vec![
                row(vec!["name ", "age"]),
                row(vec!["alice", "30"]),
                row(vec!["bob  ", "25"]),
            ],
            Level::Line,
        )));
        assert_eq!(result, expected);
    }

    #[test]
    fn columnate_varying_widths() {
        let input = Value::Array(Array::from((
            vec![
                Value::Array(Array::from((
                    vec![text("a"), text("bb"), text("ccc")],
                    Level::Word,
                ))),
                Value::Array(Array::from((
                    vec![text("dddd"), text("e"), text("ff")],
                    Level::Word,
                ))),
            ],
            Level::Line,
        )));
        let result = Columnate.apply(input).unwrap();
        let expected = Value::Array(Array::from((
            vec![
                row(vec!["a   ", "bb", "ccc"]),
                row(vec!["dddd", "e ", "ff"]),
            ],
            Level::Line,
        )));
        assert_eq!(result, expected);
    }

    #[test]
    fn columnate_single_row() {
        let input = Value::Array(Array::from((
            vec![Value::Array(Array::from((
                vec![text("one"), text("two"), text("three")],
                Level::Word,
            )))],
            Level::Line,
        )));
        let result = Columnate.apply(input).unwrap();
        let expected = Value::Array(Array::from((
            vec![row(vec!["one", "two", "three"])],
            Level::Line,
        )));
        assert_eq!(result, expected);
    }

    #[test]
    fn columnate_single_column() {
        let input = Value::Array(Array::from((
            vec![
                Value::Array(Array::from((vec![text("first")], Level::Word))),
                Value::Array(Array::from((vec![text("second")], Level::Word))),
                Value::Array(Array::from((vec![text("third")], Level::Word))),
            ],
            Level::Line,
        )));
        let result = Columnate.apply(input).unwrap();
        let expected = Value::Array(Array::from((
            vec![row(vec!["first"]), row(vec!["second"]), row(vec!["third"])],
            Level::Line,
        )));
        assert_eq!(result, expected);
    }

    #[test]
    fn columnate_empty_array() {
        let input = Value::Array(Array::from((vec![], Level::Line)));
        let result = Columnate.apply(input).unwrap();
        let expected = Value::Array(Array::from((vec![], Level::Line)));
        assert_eq!(result, expected);
    }

    #[test]
    fn columnate_with_numbers() {
        let input = Value::Array(Array::from((
            vec![
                Value::Array(Array::from((
                    vec![text("count"), text("value")],
                    Level::Word,
                ))),
                Value::Array(Array::from((
                    vec![Value::Number(42.0), text("foo")],
                    Level::Word,
                ))),
                Value::Array(Array::from((
                    vec![Value::Number(7.0), text("bar")],
                    Level::Word,
                ))),
            ],
            Level::Line,
        )));
        let result = Columnate.apply(input).unwrap();
        let expected = Value::Array(Array::from((
            vec![
                row(vec!["count", "value"]),
                row(vec!["42   ", "foo"]),
                row(vec!["7    ", "bar"]),
            ],
            Level::Line,
        )));
        assert_eq!(result, expected);
    }

    #[test]
    fn columnate_uneven_rows() {
        let input = Value::Array(Array::from((
            vec![
                Value::Array(Array::from((
                    vec![text("a"), text("b"), text("c")],
                    Level::Word,
                ))),
                Value::Array(Array::from((vec![text("d"), text("e")], Level::Word))),
                Value::Array(Array::from((vec![text("f")], Level::Word))),
            ],
            Level::Line,
        )));
        let result = Columnate.apply(input).unwrap();
        let expected = Value::Array(Array::from((
            vec![
                row(vec!["a", "b", "c"]),
                row(vec!["d", "e"]),
                row(vec!["f"]),
            ],
            Level::Line,
        )));
        assert_eq!(result, expected);
    }

    #[test]
    fn columnate_non_array_rows() {
        let input = Value::Array(Array::from((
            vec![text("hello"), text("world")],
            Level::Line,
        )));
        let result = Columnate.apply(input).unwrap();
        let expected = Value::Array(Array::from((
            vec![row(vec!["hello"]), row(vec!["world"])],
            Level::Line,
        )));
        assert_eq!(result, expected);
    }

    #[test]
    fn columnate_non_array_is_identity() {
        let input = text("hello");
        let result = Columnate.apply(input).unwrap();
        assert_eq!(result, text("hello"));
    }

    #[test]
    fn columnate_flattens_nested_arrays() {
        // Input: array of arrays of arrays (needs flattening)
        let input = Value::Array(Array::from((
            vec![
                Value::Array(Array::from((
                    vec![
                        Value::Array(Array::from((vec![text("a"), text("b")], Level::Word))),
                        Value::Array(Array::from((vec![text("cc"), text("d")], Level::Word))),
                    ],
                    Level::Line,
                ))),
                Value::Array(Array::from((
                    vec![Value::Array(Array::from((
                        vec![text("eee"), text("f")],
                        Level::Word,
                    )))],
                    Level::Line,
                ))),
            ],
            Level::File,
        )));
        let result = Columnate.apply(input).unwrap();
        let expected = Value::Array(Array::from((
            vec![
                row(vec!["a  ", "b"]),
                row(vec!["cc ", "d"]),
                row(vec!["eee", "f"]),
            ],
            Level::Line,
        )));
        assert_eq!(result, expected);
    }
}
