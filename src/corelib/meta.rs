use crate::{
    rail_machine::{Context, RailDef, RailType},
    RAIL_VERSION,
};

use RailType::*;

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        RailDef::on_state_noerr(
            "type",
            "Consume a value and produce the name (a string) of its type.",
            &[A],
            &[String],
            |quote| {
                let (thing, quote) = quote.pop();
                quote.push_string(thing.type_name())
            },
        ),
        RailDef::on_state_noerr(
            "defs",
            "Produce a list of all defined commands in the current context.",
            &[],
            &[Quote],
            |state| {
                let mut defs = state.definitions.keys().collect::<Vec<_>>();
                defs.sort();

                let defs = defs
                    .iter()
                    .fold(state.child(), |quote, def| quote.push_str(def));

                state.push_quote(defs)
            },
        ),
        // TODO: In typing, consumes of 'quote-all' should be something that means 0-to-many
        RailDef::on_state_noerr(
            "quote-all",
            "Go \"up\" a context, leaving the program's previous state as a quotation.",
            &[],
            &[Quote],
            |quote| {
                let wrapper = quote.child().replace_context(Context::Main);
                let quote = quote.replace_context(Context::Quotation {
                    parent_state: Box::new(wrapper.clone()),
                });
                wrapper.push_quote(quote)
            },
        ),
        RailDef::on_state_noerr(
            "version",
            "Produces the version of Rail currently in use.",
            &[],
            &[RailType::String],
            |quote| quote.push_str(RAIL_VERSION),
        ),
    ]
}
