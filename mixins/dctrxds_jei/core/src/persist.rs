//! The file the proof of concept keeps its typed text in.

use std::fmt;

/// The format's version, for later migrations.
const SCHEMA: u64 = 1;

/// Why a saved file could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    NotJson(String),
    WrongSchema(Option<u64>),
    MissingText,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotJson(reason) => write!(f, "not valid JSON: {reason}"),
            Self::WrongSchema(Some(found)) => {
                write!(f, "schema {found}, expected {SCHEMA}")
            }
            Self::WrongSchema(None) => write!(f, "no schema number"),
            Self::MissingText => write!(f, "no text field"),
        }
    }
}

/// The file's contents for `text`.
pub fn encode(text: &str) -> String {
    serde_json::json!({ "schema": SCHEMA, "text": text }).to_string()
}

/// The text a file holds.
pub fn decode(contents: &str) -> Result<String, DecodeError> {
    let value: serde_json::Value =
        serde_json::from_str(contents).map_err(|e| DecodeError::NotJson(e.to_string()))?;
    let schema = value.get("schema").and_then(serde_json::Value::as_u64);
    if schema != Some(SCHEMA) {
        return Err(DecodeError::WrongSchema(schema));
    }
    value
        .get("text")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or(DecodeError::MissingText)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_survives_a_round_trip_with_awkward_characters() {
        for text in [
            "",
            "abc",
            "quote \" slash \\ line \n",
            "DctrXD\u{b4}s ção 日本",
        ] {
            assert_eq!(decode(&encode(text)), Ok(text.to_owned()));
        }
    }

    #[test]
    fn the_saved_format_is_stable() {
        // A fixed literal, so a change to the format fails here and not only
        // in players' files.
        assert_eq!(
            decode(r#"{"schema":1,"text":"hello"}"#),
            Ok("hello".to_owned())
        );
    }

    #[test]
    fn broken_files_are_errors_and_not_panics() {
        assert!(matches!(decode("{"), Err(DecodeError::NotJson(_))));
        assert!(matches!(decode(""), Err(DecodeError::NotJson(_))));
        assert_eq!(decode("[]"), Err(DecodeError::WrongSchema(None)));
        assert_eq!(
            decode(r#"{"schema":2,"text":"x"}"#),
            Err(DecodeError::WrongSchema(Some(2)))
        );
        assert_eq!(decode(r#"{"schema":1}"#), Err(DecodeError::MissingText));
        assert_eq!(
            decode(r#"{"schema":1,"text":7}"#),
            Err(DecodeError::MissingText)
        );
    }
}
