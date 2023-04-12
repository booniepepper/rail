use crate::rail_machine::{RailDef, RailType};

pub fn builtins() -> Vec<RailDef<'static>> {
    // TODO: Redesign. Is this a symbol table?
    vec![RailDef::on_state(
        "?",
        "Consumes a specially-formed quote of quotes. The quote of quotes is
        evaluated by twos, with the first quote as a predicate and the second
        as an action to perform. Predicates will be evaluated until a predicate
        produces true. Only the action following the first true predicate will
        be executed. If no predicates match, no actions will be performed.",
        &[RailType::Quote],
        &[],
        |state| {
            // TODO: All conditions and all actions must have the same stack effect.
            let (options, quote) = state.stack.clone().pop_quote("?");
            let state = state.replace_stack(quote);

            let mut options = options.reverse();

            while !options.is_empty() {
                let (condition, opts) = options.pop_quote("?");
                let (action, opts) = opts.pop_quote("?");
                options = opts;

                let cond_state = condition.jailed_run_in_state(state.clone())?;
                let (success, _) = cond_state.stack.clone().pop_bool("?");

                if success {
                    return action.run_in_state(state);
                }
            }

            Ok(state)
        },
    )]
}
