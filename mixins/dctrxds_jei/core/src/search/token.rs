// Derived from Just Enough Items (https://github.com/mezz/JustEnoughItems),
// Gui/src/main/java/mezz/jei/gui/search/SearchTokenizer.java and Token.java.
// Copyright (c) 2014-2015 mezz. MIT licence: LICENSES/JustEnoughItems-MIT.txt.

//! Splits search text into words, quoted phrases, exclusions and `|`.

/// One word or phrase of the search text. Positions count characters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub exclusion: bool,
    pub operator: bool,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn operator(start: usize) -> Self {
        Self {
            text: "|".to_owned(),
            exclusion: false,
            operator: true,
            start,
            end: start + 1,
        }
    }
}

fn add_token(tokens: &mut Vec<Token>, content: &str, exclusion: bool, start: usize, end: usize) {
    let text = content.trim();
    if !text.is_empty() {
        tokens.push(Token {
            text: text.to_owned(),
            exclusion,
            operator: false,
            start,
            end,
        });
    }
}

fn is_prefix(text: &str) -> bool {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => !c.is_alphanumeric() && c != '-' && c != '|',
        _ => false,
    }
}

/// The tokens of `filter_text`, in order.
pub fn tokenize(filter_text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut inside_quotes = false;
    let mut exclusion = false;
    let mut escaped = false;
    let mut start: Option<usize> = None;
    let mut end = 0;

    for (i, c) in filter_text.chars().enumerate() {
        if escaped {
            current.push(c);
            end = i + 1;
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            start.get_or_insert(i);
            end = i + 1;
            continue;
        }
        if c == '"' {
            if inside_quotes {
                add_token(&mut tokens, &current, exclusion, start.unwrap_or(i), i + 1);
                current.clear();
                inside_quotes = false;
                exclusion = false;
                start = None;
            } else if !current.is_empty() && !is_prefix(&current) {
                add_token(&mut tokens, &current, exclusion, start.unwrap_or(i), end);
                current.clear();
                exclusion = false;
                start = None;
            } else {
                inside_quotes = true;
                start.get_or_insert(i);
                end = i + 1;
            }
            continue;
        }
        if !inside_quotes && c.is_whitespace() {
            if !current.is_empty() {
                add_token(&mut tokens, &current, exclusion, start.unwrap_or(i), end);
                current.clear();
            }
            exclusion = false;
            start = None;
            continue;
        }
        if !inside_quotes && c == '|' {
            if !current.is_empty() {
                add_token(&mut tokens, &current, exclusion, start.unwrap_or(i), end);
                current.clear();
            }
            tokens.push(Token::operator(i));
            exclusion = false;
            start = None;
            continue;
        }
        if !inside_quotes && current.is_empty() && c == '-' {
            exclusion = true;
            start = Some(i);
            end = i + 1;
            continue;
        }
        start.get_or_insert(i);
        end = i + 1;
        current.push(c);
    }
    if !current.is_empty() || inside_quotes {
        add_token(&mut tokens, &current, exclusion, start.unwrap_or(0), end);
    }
    tokens
}

/// The tokens between `|` operators. There is always at least one group.
pub fn split_by_operators(tokens: &[Token]) -> Vec<Vec<Token>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    for token in tokens {
        if token.operator {
            groups.push(std::mem::take(&mut current));
        } else {
            current.push(token.clone());
        }
    }
    groups.push(current);
    groups
}

#[cfg(test)]
#[path = "token_tests.rs"]
mod tests;
