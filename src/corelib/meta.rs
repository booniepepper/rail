use crate::rail_machine::{Context, RailDef, RailType};

use RailType::*;

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        RailDef::on_state("type", "FIXME", &[A], &[String], |quote| {
            let (thing, quote) = quote.pop();
            quote.push_string(thing.type_name())
        }),
        RailDef::on_state("defs", "FIXME", &[], &[Quote], |state| {
            let mut defs = state.definitions.keys().collect::<Vec<_>>();
            defs.sort();

            let defs = defs
                .iter()
                .fold(state.child(), |quote, def| quote.push_str(def));

            state.push_quote(defs)
        }),
        // TODO: In typing, consumes of 'quote-all' should be something that means 0-to-many
        RailDef::on_state("quote-all", "FIXME", &[], &[Quote], |quote| {
            let wrapper = quote.child().replace_context(Context::Main);
            let quote = quote.replace_context(Context::Quotation {
                parent_state: Box::new(wrapper.clone()),
            });
            wrapper.push_quote(quote)
        }),
    ]
}
