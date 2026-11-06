// Ported from Just Enough Items (https://github.com/mezz/JustEnoughItems),
// Gui/src/test/java/mezz/jei/gui/search/SearchTokenizerTest.java.
// Copyright (c) 2014-2015 mezz. MIT licence: LICENSES/JustEnoughItems-MIT.txt.

use super::*;

fn token(text: &str, exclusion: bool, start: usize, end: usize) -> Token {
    Token {
        text: text.to_owned(),
        exclusion,
        operator: false,
        start,
        end,
    }
}

#[test]
fn simple_words() {
    let tokens = tokenize("iron sword");

    assert_eq!(
        vec![token("iron", false, 0, 4), token("sword", false, 5, 10)],
        tokens
    );
}

#[test]
fn quoted_phrase() {
    let tokens = tokenize("\"iron sword\"");

    assert_eq!(vec![token("iron sword", false, 0, 12)], tokens);
}

#[test]
fn unpaired_opening_quote() {
    let tokens = tokenize("\"iron sword");

    assert_eq!(vec![token("iron sword", false, 0, 11)], tokens);
}

#[test]
fn exclusion() {
    let tokens = tokenize("-diamond");

    assert_eq!(vec![token("diamond", true, 0, 8)], tokens);
}

#[test]
fn quoted_with_exclusion() {
    let tokens = tokenize("-\"iron sword\"");

    assert_eq!(vec![token("iron sword", true, 0, 13)], tokens);
}

#[test]
fn unpaired_opening_quote_with_dash() {
    let tokens = tokenize("\"-iron sword");

    assert_eq!(vec![token("-iron sword", false, 0, 12)], tokens);
}

#[test]
fn dash_in_quoted() {
    let tokens = tokenize("\"-iron sword\"");

    assert_eq!(vec![token("-iron sword", false, 0, 13)], tokens);
}

#[test]
fn quoted_with_exclusion_with_dash_in_quoted() {
    let tokens = tokenize("-\"-iron sword\"");

    assert_eq!(vec![token("-iron sword", true, 0, 14)], tokens);
}

#[test]
fn mixed() {
    let tokens = tokenize("sword \"iron pick\" -wood");

    assert_eq!(
        vec![
            token("sword", false, 0, 5),
            token("iron pick", false, 6, 17),
            token("wood", true, 18, 23)
        ],
        tokens
    );
}

#[test]
fn empty_and_whitespace() {
    assert!(tokenize("").is_empty());
    assert!(tokenize("   ").is_empty());
    assert!(tokenize("-   ").is_empty());
    assert!(tokenize(" \"  ").is_empty());
}

#[test]
fn only_quotes() {
    let tokens = tokenize("\"\"");

    assert!(tokens.is_empty());
}

#[test]
fn escaped_quote_can_be_searched_in_unquoted_token() {
    let tokens = tokenize("iron\\\"sword \\\"quoted\\\"");

    assert_eq!(
        vec![
            token("iron\"sword", false, 0, 11),
            token("\"quoted\"", false, 12, 22)
        ],
        tokens
    );
}

#[test]
fn escaped_quote_can_be_searched_in_quoted_phrase() {
    let tokens = tokenize("\"the \\\"best\\\" sword\"");

    assert_eq!(vec![token("the \"best\" sword", false, 0, 20)], tokens);
}

#[test]
fn escaped_quote_can_be_searched_in_excluded_quoted_phrase() {
    let tokens = tokenize("-\"the \\\"best\\\" sword\"");

    assert_eq!(vec![token("the \"best\" sword", true, 0, 21)], tokens);
}

#[test]
fn escaped_quote_can_be_searched_in_prefixed_quoted_phrase() {
    let filter_text = concat!("$\"tooltip says \\\"place\\\"\" ", "-@\"mod \\\"name\\\"\"");
    let tokens = tokenize(filter_text);

    assert_eq!(
        vec![
            token("$tooltip says \"place\"", false, 0, 25),
            token("@mod \"name\"", true, 26, 42)
        ],
        tokens
    );
}

#[test]
fn quoted_phrase_trims_whitespace_inside_quotes() {
    let tokens = tokenize("\"  iron sword  \" -\"  gold ingot  \"");

    assert_eq!(
        vec![
            token("iron sword", false, 0, 16),
            token("gold ingot", true, 17, 34)
        ],
        tokens
    );
}

#[test]
fn whitespace_characters_separate_tokens_outside_quotes() {
    let tokens = tokenize("iron\t\"gold ingot\"\n-wood\r\n\"red\tstone\"");

    assert_eq!(
        vec![
            token("iron", false, 0, 4),
            token("gold ingot", false, 5, 17),
            token("wood", true, 18, 23),
            token("red\tstone", false, 25, 36)
        ],
        tokens
    );
}

#[test]
fn prefixed_quoted_phrase() {
    let tokens = tokenize("%\"redstone blocks\"");

    assert_eq!(vec![token("%redstone blocks", false, 0, 18)], tokens);
}

#[test]
fn prefixed_quoted_phrase_with_exclusion() {
    let tokens = tokenize("-%\"redstone blocks\"");

    assert_eq!(vec![token("%redstone blocks", true, 0, 19)], tokens);
}

#[test]
fn each_search_prefix_supports_quoted_phrases() {
    let filter_text = "@\"just enough items\" #\"minecraft logs\" $\"tooltip text\" %\"redstone blocks\" ^\"light blue\" &\"minecraft:iron_ingot\"";
    let tokens = tokenize(filter_text);

    assert_eq!(
        vec![
            token("@just enough items", false, 0, 20),
            token("#minecraft logs", false, 21, 38),
            token("$tooltip text", false, 39, 54),
            token("%redstone blocks", false, 55, 73),
            token("^light blue", false, 74, 87),
            token("&minecraft:iron_ingot", false, 88, 111)
        ],
        tokens
    );
}

#[test]
fn each_search_prefix_supports_excluded_quoted_phrases() {
    let filter_text = "-@\"just enough items\" -#\"minecraft logs\" -$\"tooltip text\" -%\"redstone blocks\" -^\"light blue\" -&\"minecraft:iron_ingot\"";
    let tokens = tokenize(filter_text);

    assert_eq!(
        vec![
            token("@just enough items", true, 0, 21),
            token("#minecraft logs", true, 22, 40),
            token("$tooltip text", true, 41, 57),
            token("%redstone blocks", true, 58, 77),
            token("^light blue", true, 78, 92),
            token("&minecraft:iron_ingot", true, 93, 117)
        ],
        tokens
    );
}

#[test]
fn unpaired_opening_quote_after_prefix() {
    let tokens = tokenize("%\"redstone blocks");

    assert_eq!(vec![token("%redstone blocks", false, 0, 17)], tokens);
}

#[test]
fn bare_exclusion_marker_does_not_exclude_next_token() {
    let tokens = tokenize("- diamond - \"iron sword\"");

    assert_eq!(
        vec![
            token("diamond", false, 2, 9),
            token("iron sword", false, 12, 24)
        ],
        tokens
    );
}

#[test]
fn empty_quoted_exclusions_do_not_affect_next_token() {
    let tokens = tokenize("-\"\" diamond -\"  \" gold");

    assert_eq!(
        vec![token("diamond", false, 4, 11), token("gold", false, 18, 22)],
        tokens
    );
}

#[test]
fn dash_inside_unquoted_token_is_not_exclusion() {
    let tokens = tokenize("iron-sword @mod-name %-redstone");

    assert_eq!(
        vec![
            token("iron-sword", false, 0, 10),
            token("@mod-name", false, 11, 20),
            token("%-redstone", false, 21, 31)
        ],
        tokens
    );
}

#[test]
fn repeated_tokens() {
    let tokens = tokenize("diamond diamond diamond diamond");

    assert_eq!(
        vec![
            token("diamond", false, 0, 7),
            token("diamond", false, 8, 15),
            token("diamond", false, 16, 23),
            token("diamond", false, 24, 31)
        ],
        tokens
    );
}

#[test]
fn or_operator() {
    let tokens = tokenize("iron|sword");

    assert_eq!(
        vec![
            token("iron", false, 0, 4),
            Token::operator(4),
            token("sword", false, 5, 10)
        ],
        tokens
    );
}

#[test]
fn or_operator_with_spaces() {
    let tokens = tokenize("iron | sword");

    assert_eq!(
        vec![
            token("iron", false, 0, 4),
            Token::operator(5),
            token("sword", false, 7, 12)
        ],
        tokens
    );
}

#[test]
fn or_operator_inside_quotes() {
    let tokens = tokenize("\"iron|sword\"");

    assert_eq!(vec![token("iron|sword", false, 0, 12)], tokens);
}

#[test]
fn escaped_or_operator_is_not_operator() {
    let tokens = tokenize("iron\\|sword");

    assert_eq!(vec![token("iron|sword", false, 0, 11)], tokens);
}

#[test]
fn literal_pipe_is_not_operator() {
    let quoted = tokenize("\"|\"");
    assert_eq!(vec![token("|", false, 0, 3)], quoted);

    let escaped = tokenize("\\|");
    assert_eq!(vec![token("|", false, 0, 2)], escaped);
}

#[test]
fn or_operator_with_exclusion() {
    let tokens = tokenize("-iron|sword");

    assert_eq!(
        vec![
            token("iron", true, 0, 5),
            Token::operator(5),
            token("sword", false, 6, 11)
        ],
        tokens
    );
}

#[test]
fn multiple_or_operators() {
    let tokens = tokenize("a|b|c");

    assert_eq!(
        vec![
            token("a", false, 0, 1),
            Token::operator(1),
            token("b", false, 2, 3),
            Token::operator(3),
            token("c", false, 4, 5)
        ],
        tokens
    );
}

#[test]
fn leading_and_trailing_or() {
    let leading = tokenize("|iron");
    assert_eq!(
        vec![Token::operator(0), token("iron", false, 1, 5)],
        leading
    );

    let trailing = tokenize("iron|");
    assert_eq!(
        vec![token("iron", false, 0, 4), Token::operator(4)],
        trailing
    );
}

#[test]
fn split_by_operators() {
    let tokens = vec![
        token("iron", false, 0, 4),
        Token::operator(4),
        token("sword", false, 5, 10),
        Token::operator(10),
        token("gold", false, 11, 15),
    ];

    let groups = super::split_by_operators(&tokens);
    assert_eq!(groups.len(), 3);
    assert_eq!(vec![token("iron", false, 0, 4)], groups[0]);
    assert_eq!(vec![token("sword", false, 5, 10)], groups[1]);
    assert_eq!(vec![token("gold", false, 11, 15)], groups[2]);
}

#[test]
fn split_by_operators_without_operators() {
    let tokens = vec![token("iron", false, 0, 4), token("sword", false, 5, 10)];

    let groups = super::split_by_operators(&tokens);
    assert_eq!(groups.len(), 1);
    assert_eq!(tokens, groups[0]);
}

#[test]
fn split_by_operators_empty() {
    let groups = super::split_by_operators(&[]);
    assert_eq!(groups.len(), 1);
    assert!(groups[0].is_empty());
}

#[test]
fn unpaired_quote_separates_words() {
    let tokens = tokenize("iron\"sword");

    assert_eq!(
        vec![token("iron", false, 0, 4), token("sword", false, 5, 10)],
        tokens
    );
}

#[test]
fn unpaired_quote_at_end_does_not_produce_empty_token() {
    let tokens = tokenize("iron\"");

    assert_eq!(vec![token("iron", false, 0, 4)], tokens);
}

#[test]
fn prefix_before_unpaired_quote_starts_quoted_phrase() {
    let tokens = tokenize("%\"redstone blocks");

    assert_eq!(vec![token("%redstone blocks", false, 0, 17)], tokens);
}

#[test]
fn adjacent_quoted_phrases() {
    let tokens = tokenize("\"a\"\"b\"");

    assert_eq!(
        vec![token("a", false, 0, 3), token("b", false, 3, 6)],
        tokens
    );
}
