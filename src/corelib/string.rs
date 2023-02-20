use crate::rail_machine::{RailDef, RailType, Stack};

use RailType::*;

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        RailDef::on_state_noerr("upcase", "Consume a string and produce an identical string in all uppercase.", &[String], &[String], |quote| {
            let (s, quote) = quote.pop_string("upcase");
            quote.push_string(s.to_uppercase())
        }),
        RailDef::on_state_noerr("downcase", "Consume a string and produce an identical string in all lowercase.", &[String], &[String], |quote| {
            let (s, quote) = quote.pop_string("downcase");
            quote.push_string(s.to_lowercase())
        }),
        RailDef::on_state_noerr("trim", "Consume a string and produce an identical string with all leading and trailing whitespace removed.", &[String], &[String], |quote| {
            let (s, quote) = quote.pop_string("trim");
            quote.push_string(s.trim().to_string())
        }),
        // TODO: Should this also work on Quotes?
        RailDef::on_state_noerr("split", "Consume a string and a string as a separator. Produce a list of terms from the first string split on the separator.", &[String, String], &[Quote], |state| {
            let (delimiter, state) = state.pop_string("split");
            let (s, state) = state.pop_string("split");

            let words = s
                .split(&delimiter)
                .fold(state.child(), |words, word| words.push_str(word));

            state.push_quote(words)
        }),
        // TODO: Should this also work on Quotes?
        RailDef::on_state_noerr("join", "Consume a list of strings and a string separator. Produce a string with all strings in the original list joined with the separator.", &[Quote, String], &[String], |quote| {
            let (delimiter, quote) = quote.pop_string("join");
            let (strings, quote) = quote.pop_quote("join");
            quote.push_string(join("join", strings.stack, &delimiter))
        }),
        RailDef::on_state_noerr(
            "contains?",
            "Consume one string and one string as a substring. Produce true if the substring occurs in the original string, and false otherwise.",
            &[String, String],
            &[Boolean],
            |quote| {
                let (substring, quote) = quote.pop_string("contains?");
                let (string, quote) = quote.pop_string("contains?");
                let is_contained = string.contains(&substring);
                quote.push_bool(is_contained)
            },
        ),
        RailDef::on_state_noerr(
            "starts-with?",
            "Consume one string and one string as a substring. Produce true if the substring is a prefix of the original string, and false otherwise.",
            &[String, String],
            &[Boolean],
            |quote| {
                let (prefix, quote) = quote.pop_string("starts-with?");
                let (string, quote) = quote.pop_string("starts-with?");
                let is_prefix = string.starts_with(&prefix);
                quote.push_bool(is_prefix)
            },
        ),
        RailDef::on_state_noerr(
            "ends-with?",
            "Consume one string and one string as a substring. Produce true if the substring is a suffix of the original string, and false otherwise.",
            &[String, String],
            &[Boolean],
            |quote| {
                let (suffix, quote) = quote.pop_string("ends-with?");
                let (string, quote) = quote.pop_string("ends-with?");
                let is_suffix = string.ends_with(&suffix);
                quote.push_bool(is_suffix)
            },
        ),
        RailDef::on_state_noerr("to-string", "Consume one value and produce a string representation of it.", &[A], &[String], |quote| {
            let (a, quote) = quote.pop();
            let a = format!("{}", a);
            quote.push_string(a)
        }),
    ]
}

fn join(context: &str, words: Stack, delimiter: &str) -> std::string::String {
    let mut s = vec![];
    let mut words = words;
    while !words.is_empty() {
        let (part, new_words) = words.pop_string(context);
        s.push(part);
        words = new_words
    }
    s.reverse();
    s.join(delimiter)
}
