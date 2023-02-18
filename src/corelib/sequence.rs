use crate::log;
use crate::rail_machine::{RailDef, RailType, RailVal, Stack};

use RailType::*;

// TODO: These should all work for both String and Quote? Should String also be a Quote? Typeclasses?
pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        RailDef::on_state("len", "Consume a quote or string, and produce its length.", &[QuoteOrString], &[I64], |quote| {
            let (a, quote) = quote.pop();
            let len: i64 = match a {
                RailVal::Quote(quote) => quote.len(),
                RailVal::String(s) => s.len(),
                _ => {
                    log::warn(
                        quote.conventions,
                        format!("Can only perform len on quote or string but got {}", a),
                    );
                    return quote.push(a);
                }
            }
            .try_into()
            .unwrap();
            quote.push_i64(len)
        }),
        RailDef::on_state("quote", "Consume a value, and produce a quote containing only that value.", &[A], &[Quote], |state| {
            let (a, state) = state.pop();
            let quote = state.child().push(a);
            state.push_quote(quote)
        }),
        RailDef::on_state("unquote", "Consume a quote, and produce its values.", &[Quote], &[Unknown], |state| {
            let (quote, mut state) = state.pop_quote("unquote");
            for value in quote.stack.values {
                state = state.push(value);
            }
            state
        }),
        RailDef::on_state("as-quote", "Consume a value. If it is already a quote, produce it. Otherwise, produce a quote containing only that value.", &[A], &[Quote], |state| {
            let (a, state) = state.pop();
            let quote = match a {
                RailVal::Quote(quote) => quote,
                _ => state.child().push(a),
            };
            state.push_quote(quote)
        }),
        RailDef::on_state("push", "Consume a quote and a value, and produce an identical quote with the value appended.", &[Quote, A], &[Quote], |quote| {
            let (a, quote) = quote.pop();
            let (sequence, quote) = quote.pop_quote("push");
            let sequence = sequence.push(a);
            quote.push_quote(sequence)
        }),
        RailDef::on_state("pop", "Consume a quote, and produce a quote (with the last element removed) and the quote's last element.", &[Quote], &[Quote, A], |quote| {
            let (sequence, quote) = quote.pop_quote("pop");
            let (a, sequence) = sequence.pop();
            quote.push_quote(sequence).push(a)
        }),
        RailDef::on_state("enq", "Consume a value and a quote, and produce an identical quote with the value prepended.", &[A, Quote], &[Quote], |quote| {
            let (sequence, quote) = quote.pop_quote("push");
            let (a, quote) = quote.pop();
            let sequence = sequence.enqueue(a);
            quote.push_quote(sequence)
        }),
        RailDef::on_state("nth", "Consume a quote and an integer, and produce the element at the 0-indexed location specified.", &[Quote, I64], &[A], |state| {
            let (nth, state) = state.pop_i64("nth");
            let (seq, state) = state.pop_quote("nth");

            let nth = seq.stack.values.get(nth as usize).unwrap();

            state.push(nth.clone())
        }),
        RailDef::on_state("deq", "Consume a quote, and produce its first value and the quote with the first value removed.", &[Quote], &[A, Quote], |quote| {
            let (sequence, quote) = quote.pop_quote("pop");
            let (a, sequence) = sequence.dequeue();
            quote.push(a).push_quote(sequence)
        }),
        RailDef::on_state("rev", "Consume a quote or string, and produce its reverse.", &[QuoteOrString], &[Quote], |quote| {
            let (a, quote) = quote.pop();
            match a {
                RailVal::String(s) => quote.push_string(s.chars().rev().collect()),
                RailVal::Quote(q) => quote.push_quote(q.reverse()),
                _ => {
                    log::warn(
                        quote.conventions,
                        format!("Can only perform len on quote or string but got {}", a),
                    );
                    quote.push(a)
                }
            }
        }),
        RailDef::on_state("concat", "Consume two quotes or two strings and produce their concatenated value.", &[QuoteOrString, QuoteOrString], &[QuoteOrString], |quote| {
            let (suffix, quote) = quote.pop();
            let (prefix, quote) = quote.pop();

            match (&prefix, &suffix) {
                (RailVal::String(p), RailVal::String(s)) => quote.push_string(p.to_owned() + s),
                (RailVal::Quote(prefix), RailVal::Quote(suffix)) => {
                    let mut results = quote.child();
                    for term in prefix.clone().stack.values.into_iter().chain(suffix.clone().stack.values) {
                        results = results.push(term);
                    }
                    quote.push_quote(results)
                }
                _ => {
                    log::warn(
                        quote.conventions,
                        format!("Can only perform concat when previous two values are both strings or both quotes. Instead got {} and {}", prefix, suffix),
                    );
                    quote.push(prefix).push(suffix)
                }
            }
        }),
        RailDef::on_state("filter", "Consume one quote as a list and another quote as a predicate, produce a list of all values from the original list that return true for the predicate.", &[Quote, Quote], &[Quote], |state| {
            let (predicate, state) = state.pop_quote("filter");
            let (sequence, state) = state.pop_quote("filter");
            let mut results = state.child();

            for term in sequence.stack.values {
                let substate = state.child().replace_stack(Stack::of(term.clone()));
                let substate = predicate.clone().jailed_run_in_state(substate);
                let (keep, _) = substate.stack.pop_bool("filter");
                if keep {
                    results = results.push(term);
                }
            }

            state.push_quote(results)
        }),
        RailDef::on_state("map", "Consume one quote as a list and another quote as a transform, produce a list of all values from the original list after applying the transformation.", &[Quote, Quote], &[Quote], |state| {
            let (transform, state) = state.pop_quote("map");
            let (sequence, state) = state.pop_quote("map");

            let mut results = state.child();

            for term in sequence.stack.values {
                results = results.push(term.clone());
                let substate = state.child().replace_stack(results.stack);
                let substate = transform.clone().jailed_run_in_state(substate);
                results = substate;
            }

            state.push_quote(results)
        }),
        RailDef::on_state("each!", "Consume one quote as a list and another quote as commands. Run the commands on each list, any definitions will be preserved in the calling context.", &[Quote, Quote], &[Unknown], |state| {
            let (command, state) = state.pop_quote("each!");
            let (sequence, state) = state.pop_quote("each!");

            sequence
                .stack
                .values
                .into_iter()
                .fold(state, |state, value| {
                    let state = state.update_stack(|quote| quote.push(value.clone()));
                    command.clone().run_in_state(state)
                })
        }),
        RailDef::on_jailed_state("each", "Consume one quote as a list and another quote as commands. Run the commands on each list, any definitions will NOT be preserved in the calling context.", &[Quote, Quote], &[Unknown], |state| {
            let (command, state) = state.pop_quote("each");
            let (sequence, state) = state.pop_quote("each");

            let definitions = state.definitions.clone();

            sequence
                .stack
                .values
                .into_iter()
                .fold(state, |state, value| {
                    let state = state
                        .update_stack(|quote| quote.push(value.clone()))
                        .replace_definitions(definitions.clone());
                    command.clone().jailed_run_in_state(state)
                })
        }),
        RailDef::on_state("zip", "Consume two quotes as lists, and produce a list of pairs of values. The result as short as the shortest list; additional values from a longer list will be discarded.", &[Quote, Quote], &[Quote], |state| {
            let (b, state) = state.pop_quote("zip");
            let (a, state) = state.pop_quote("zip");

            let c = a
                .stack
                .values
                .into_iter()
                .zip(b.stack.values)
                .map(|(a, b)| state.child().push(a).push(b))
                .fold(state.child(), |c, quote| c.push_quote(quote));

            state.push_quote(c)
        }),
        RailDef::on_state(
            "zip-with",
            "Consume two quotes as lists and one quote as a list of commands. Produce a list of values that are the pairwise application of the commands to values in the original list. The result as short as the shortest list; additional values from a longer list will be discarded.",
            &[Quote, Quote, Quote],
            &[Quote],
            |state| {
                let (xform, state) = state.pop_quote("zip-with");
                let (b, state) = state.pop_quote("zip-with");
                let (a, state) = state.pop_quote("zip-with");

                let c = a
                    .stack
                    .values
                    .into_iter()
                    .zip(b.stack.values)
                    .map(|(a, b)| state.child().push(a).push(b))
                    .map(|ab| xform.clone().run_in_state(ab))
                    .fold(state.child(), |c, result| c.push_quote(result));

                state.push_quote(c)
            },
        ),
    ]
}
