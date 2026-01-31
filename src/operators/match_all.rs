use regex::Regex;

use crate::error::Result;
use crate::interpreter::Transform;
use crate::value::{Array, Level, Value};

pub struct MatchAll {
    pattern: Regex,
}

impl MatchAll {
    pub fn new(pattern: Regex) -> Self {
        Self { pattern }
    }

    fn extract_matches(&self, text: &str) -> Vec<Value> {
        self.pattern
            .find_iter(text)
            .map(|m| Value::Text(m.as_str().to_string()))
            .collect()
    }
}

impl Transform for MatchAll {
    fn apply(&self, value: Value) -> Result<Value> {
        match value {
            Value::Array(arr) => {
                let results: Vec<Value> = arr
                    .elements
                    .into_iter()
                    .map(|elem| {
                        let text = match &elem {
                            Value::Text(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Array(inner) => inner.to_string(),
                        };
                        let matches = self.extract_matches(&text);
                        Value::Array(Array::from((matches, Level::Word)))
                    })
                    .collect();
                Ok(Value::Array(Array::from((results, arr.level))))
            }
            Value::Text(s) => {
                let matches = self.extract_matches(&s);
                Ok(Value::Array(Array::from((matches, Level::Word))))
            }
            Value::Number(n) => {
                let matches = self.extract_matches(&n.to_string());
                Ok(Value::Array(Array::from((matches, Level::Word))))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Value {
        Value::Text(s.to_string())
    }

    fn arr(elements: Vec<Value>, level: Level) -> Value {
        Value::Array(Array::from((elements, level)))
    }

    #[test]
    fn match_single_string() {
        let input = text("hello world hello");
        let matcher = MatchAll::new(Regex::new("hello").unwrap());
        let result = matcher.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("hello"));
                assert_eq!(arr.elements[1], text("hello"));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn match_no_matches() {
        let input = text("hello world");
        let matcher = MatchAll::new(Regex::new("xyz").unwrap());
        let result = matcher.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert!(arr.is_empty());
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn match_array_of_strings() {
        let input = arr(vec![text("foo bar foo"), text("baz foo")], Level::Line);
        let matcher = MatchAll::new(Regex::new("foo").unwrap());
        let result = matcher.apply(input).unwrap();

        match result {
            Value::Array(outer) => {
                assert_eq!(outer.len(), 2);
                assert_eq!(outer.level, Level::Line);

                match &outer.elements[0] {
                    Value::Array(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(inner.elements[0], text("foo"));
                        assert_eq!(inner.elements[1], text("foo"));
                    }
                    _ => panic!("expected inner array"),
                }

                match &outer.elements[1] {
                    Value::Array(inner) => {
                        assert_eq!(inner.len(), 1);
                        assert_eq!(inner.elements[0], text("foo"));
                    }
                    _ => panic!("expected inner array"),
                }
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn match_ip_addresses() {
        let input = arr(
            vec![
                text("192.168.1.1 connected to 10.0.0.1"),
                text("172.16.0.1 only"),
            ],
            Level::Line,
        );
        let matcher = MatchAll::new(Regex::new(r"\d+\.\d+\.\d+\.\d+").unwrap());
        let result = matcher.apply(input).unwrap();

        match result {
            Value::Array(outer) => {
                assert_eq!(outer.len(), 2);

                match &outer.elements[0] {
                    Value::Array(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(inner.elements[0], text("192.168.1.1"));
                        assert_eq!(inner.elements[1], text("10.0.0.1"));
                    }
                    _ => panic!("expected inner array"),
                }

                match &outer.elements[1] {
                    Value::Array(inner) => {
                        assert_eq!(inner.len(), 1);
                        assert_eq!(inner.elements[0], text("172.16.0.1"));
                    }
                    _ => panic!("expected inner array"),
                }
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn match_with_numbers() {
        let input = arr(vec![Value::Number(12345.0)], Level::Line);
        let matcher = MatchAll::new(Regex::new(r"\d").unwrap());
        let result = matcher.apply(input).unwrap();

        match result {
            Value::Array(outer) => match &outer.elements[0] {
                Value::Array(inner) => {
                    assert_eq!(inner.len(), 5);
                    assert_eq!(inner.elements[0], text("1"));
                    assert_eq!(inner.elements[4], text("5"));
                }
                _ => panic!("expected inner array"),
            },
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn match_preserves_outer_level() {
        let input = arr(vec![text("foo")], Level::Word);
        let matcher = MatchAll::new(Regex::new("foo").unwrap());
        let result = matcher.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.level, Level::Word);
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn match_empty_array() {
        let input = arr(vec![], Level::Line);
        let matcher = MatchAll::new(Regex::new("foo").unwrap());
        let result = matcher.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert!(arr.is_empty());
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn match_extract_numbers() {
        let input = text("price: $42, quantity: 7");
        let matcher = MatchAll::new(Regex::new(r"\d+").unwrap());
        let result = matcher.apply(input).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr.elements[0], text("42"));
                assert_eq!(arr.elements[1], text("7"));
            }
            _ => panic!("expected array"),
        }
    }
}
