//! The files the mod keeps: bookmarks, hidden ingredients and lookup history.
//! Each holds a `schema` number, for later migrations. Reading a broken file is
//! an error, never a panic: the adapter keeps it as a backup and starts empty.

use crate::ingredient::Stack;
use serde_json::{Value, json};
use std::fmt;

const SCHEMA: u64 = 1;

/// Why a saved file could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    NotJson(String),
    WrongSchema(Option<u64>),
    MissingField(&'static str),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotJson(reason) => write!(f, "not valid JSON: {reason}"),
            Self::WrongSchema(Some(found)) => write!(f, "schema {found}, expected {SCHEMA}"),
            Self::WrongSchema(None) => write!(f, "no schema number"),
            Self::MissingField(field) => write!(f, "no field {field}"),
        }
    }
}

/// A saved place in the bookmark list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bookmark {
    Ingredient(Stack),
    Recipe { id: String },
}

fn stack_json(stack: &Stack) -> Value {
    if stack.components.is_empty() {
        json!({ "item": stack.item })
    } else {
        json!({ "item": stack.item, "components": stack.components })
    }
}

fn stack_from(value: &Value) -> Option<Stack> {
    let item = value.get("item")?.as_str()?;
    let components = value
        .get("components")
        .and_then(Value::as_str)
        .unwrap_or_default();
    Some(Stack::with_components(item, components))
}

fn open<'a>(
    contents: &str,
    value: &'a mut Value,
    field: &'static str,
) -> Result<&'a [Value], DecodeError> {
    *value = serde_json::from_str(contents).map_err(|e| DecodeError::NotJson(e.to_string()))?;
    let schema = value.get("schema").and_then(Value::as_u64);
    if schema != Some(SCHEMA) {
        return Err(DecodeError::WrongSchema(schema));
    }
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or(DecodeError::MissingField(field))
}

pub fn encode_bookmarks(bookmarks: &[Bookmark]) -> String {
    let list: Vec<Value> = bookmarks
        .iter()
        .map(|bookmark| match bookmark {
            Bookmark::Ingredient(stack) => {
                let mut value = stack_json(stack);
                value["type"] = json!("ingredient");
                value
            }
            Bookmark::Recipe { id } => json!({ "type": "recipe", "id": id }),
        })
        .collect();
    json!({ "schema": SCHEMA, "bookmarks": list }).to_string()
}

/// The bookmarks a file holds. An entry that can't be read is left out, so one
/// bad line doesn't lose the others.
pub fn decode_bookmarks(contents: &str) -> Result<Vec<Bookmark>, DecodeError> {
    let mut value = Value::Null;
    let list = open(contents, &mut value, "bookmarks")?;
    Ok(list
        .iter()
        .filter_map(|entry| match entry.get("type").and_then(Value::as_str)? {
            "ingredient" => stack_from(entry).map(Bookmark::Ingredient),
            "recipe" => Some(Bookmark::Recipe {
                id: entry.get("id")?.as_str()?.to_owned(),
            }),
            _ => None,
        })
        .collect())
}

/// A plain list of stacks: the hidden ingredients and the lookup history.
pub fn encode_stacks(stacks: &[Stack]) -> String {
    let list: Vec<Value> = stacks.iter().map(stack_json).collect();
    json!({ "schema": SCHEMA, "stacks": list }).to_string()
}

pub fn decode_stacks(contents: &str) -> Result<Vec<Stack>, DecodeError> {
    let mut value = Value::Null;
    let list = open(contents, &mut value, "stacks")?;
    Ok(list.iter().filter_map(stack_from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmarks_survive_a_round_trip_with_awkward_characters() {
        let bookmarks = vec![
            Bookmark::Ingredient(Stack::of("m:stone")),
            Bookmark::Ingredient(Stack::with_components(
                "m:potion",
                "[a={potion:\"m:swift\\\\\"},ç=日本]",
            )),
            Bookmark::Recipe {
                id: "m:planks".into(),
            },
        ];
        assert_eq!(
            decode_bookmarks(&encode_bookmarks(&bookmarks)),
            Ok(bookmarks)
        );
    }

    #[test]
    fn the_saved_format_is_stable() {
        // Fixed literals, so a change to the format fails here and not only in
        // players' files.
        assert_eq!(
            decode_bookmarks(
                r#"{"schema":1,"bookmarks":[{"type":"ingredient","item":"m:a"},{"type":"recipe","id":"m:r"}]}"#
            ),
            Ok(vec![
                Bookmark::Ingredient(Stack::of("m:a")),
                Bookmark::Recipe { id: "m:r".into() }
            ])
        );
        assert_eq!(
            decode_stacks(r#"{"schema":1,"stacks":[{"item":"m:a","components":"[x]"}]}"#),
            Ok(vec![Stack::with_components("m:a", "[x]")])
        );
    }

    #[test]
    fn broken_files_are_errors_and_not_panics() {
        assert!(matches!(
            decode_bookmarks("{"),
            Err(DecodeError::NotJson(_))
        ));
        assert!(matches!(decode_stacks(""), Err(DecodeError::NotJson(_))));
        assert_eq!(decode_bookmarks("[]"), Err(DecodeError::WrongSchema(None)));
        assert_eq!(
            decode_bookmarks(r#"{"schema":2,"bookmarks":[]}"#),
            Err(DecodeError::WrongSchema(Some(2)))
        );
        assert_eq!(
            decode_bookmarks(r#"{"schema":1}"#),
            Err(DecodeError::MissingField("bookmarks"))
        );
    }

    #[test]
    fn a_bad_entry_does_not_lose_the_others() {
        let contents = r#"{"schema":1,"bookmarks":[{"type":"other"},{"type":"recipe"},7,{"type":"ingredient","item":"m:a"}]}"#;
        assert_eq!(
            decode_bookmarks(contents),
            Ok(vec![Bookmark::Ingredient(Stack::of("m:a"))])
        );
    }

    #[test]
    fn errors_say_what_is_wrong() {
        assert_eq!(
            DecodeError::WrongSchema(Some(2)).to_string(),
            "schema 2, expected 1"
        );
        assert_eq!(DecodeError::MissingField("x").to_string(), "no field x");
    }
}
