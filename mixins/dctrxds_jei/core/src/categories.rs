//! Categories other mods declare with data, with no Rust: a JSON file in
//! `data/<namespace>/dctrxds_jei/categories/<name>.json`:
//!
//! ```json
//! { "title": "Crushing", "icon": "mymod:crusher", "catalysts": ["mymod:crusher"],
//!   "recipe_type": "mymod:crushing", "inputs": ["ingredient"], "outputs": ["result"] }
//! ```
//!
//! `inputs` and `outputs` name the fields of that recipe type's JSON.

use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CategoryDecl {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub catalysts: Vec<String>,
    pub recipe_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

fn text(value: &Value, field: &str) -> Result<String, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|t| !t.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{field}: needs a text"))
}

fn texts(value: &Value, field: &str) -> Result<Vec<String>, String> {
    match value.get(field) {
        None => Ok(Vec::new()),
        Some(list) => list
            .as_array()
            .and_then(|list| {
                list.iter()
                    .map(|v| v.as_str().map(str::to_owned))
                    .collect::<Option<Vec<_>>>()
            })
            .ok_or_else(|| format!("{field}: needs a list of texts")),
    }
}

impl CategoryDecl {
    /// Reads one file; `id` is `<namespace>:<name>`, from its path.
    pub fn parse(id: &str, value: &Value) -> Result<Self, String> {
        let decl = Self {
            id: id.to_owned(),
            title: text(value, "title")?,
            icon: text(value, "icon")?,
            catalysts: texts(value, "catalysts")?,
            recipe_type: text(value, "recipe_type")?,
            inputs: texts(value, "inputs")?,
            outputs: texts(value, "outputs")?,
        };
        if decl.inputs.is_empty() && decl.outputs.is_empty() {
            return Err("inputs, outputs: name at least one field".to_owned());
        }
        Ok(decl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_complete_file_is_read() {
        let file = json!({
            "title": "Crushing", "icon": "m:crusher", "catalysts": ["m:crusher"],
            "recipe_type": "m:crushing", "inputs": ["ingredient"], "outputs": ["result"]
        });
        let decl = CategoryDecl::parse("m:crush", &file).unwrap();
        assert_eq!(decl.id, "m:crush");
        assert_eq!(decl.inputs, ["ingredient"]);
        assert_eq!(decl.catalysts, ["m:crusher"]);
    }

    #[test]
    fn missing_or_wrong_fields_say_which() {
        let error = |file: Value| CategoryDecl::parse("m:x", &file).unwrap_err();
        assert!(error(json!({})).contains("title"));
        assert!(
            error(json!({"title": " ", "icon": "m:a", "recipe_type": "m:b"})).contains("title")
        );
        assert!(error(json!({"title": "T", "icon": "m:a"})).contains("recipe_type"));
        assert!(
            error(json!({"title": "T", "icon": "m:a", "recipe_type": "m:b", "inputs": 3}))
                .contains("inputs")
        );
        assert!(
            error(json!({"title": "T", "icon": "m:a", "recipe_type": "m:b"}))
                .contains("name at least")
        );
    }
}
