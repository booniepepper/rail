use crate::rail_machine::{self, RailDef, RailState, RailType, RailVal};

use RailType::*;

const DEFINITIONS_PRESERVED: &str = "Any new definitions are preserved in the calling context.";
const DEFINITIONS_LOCALY_ONLY: &str =
    "Any definitions are local only to the quote or command performed.";

pub fn builtins() -> Vec<RailDef<'static>> {
    vec![
        RailDef::on_state("do!", &format!("Consumes a quote or command and executes it, producing any output(s) of the quote or command. {}", DEFINITIONS_PRESERVED), &[QuoteOrCommand], &[Unknown], do_it()),
        RailDef::on_jailed_state("do", &format!("Consumes a quote or command and executes it, producing any output(s) of the quote or command. {}", DEFINITIONS_LOCALY_ONLY), &[QuoteOrCommand], &[Unknown], do_it()),
        RailDef::on_state(
            "doin!",
            &format!("Consumes one quote and one quote or command. The latter quote or command is executed inside the first quote, producing any output(s) of the quote or command inside it. {}", DEFINITIONS_PRESERVED),
            &[Quote, QuoteOrCommand],
            &[Unknown],
            doin(),
        ),
        RailDef::on_jailed_state(
            "doin",
            &format!("Consumes one quote and one quote or command. The latter quote or command is executed inside the first quote, producing any output(s) of the quote or command inside it. {}", DEFINITIONS_LOCALY_ONLY),
            &[Quote, QuoteOrCommand],
            &[Unknown],
            doin(),
        ),
        RailDef::on_state("def!", &format!("{} {}", "Consumes one quote and a quoted command or string. The latter quoted command or string becomes a command that executes the first quote.", DEFINITIONS_PRESERVED), &[Quote, QuoteOrCommand], &[], |state| {
            let conventions = state.conventions;
            state.update_stack_and_defs(|quote, definitions| {
                let mut definitions = definitions;

                let (name, quote) = quote.pop();
                let name = if let Some(name) = get_command_name(&name) {
                    name
                } else {
                    rail_machine::log_warn(
                        conventions,
                        format!("{} is not a string or command", name),
                    );
                    return (quote, definitions);
                };

                // FIXME: Should be from the quote.
                let description = "FIXME: Undocumented";

                let (commands, quote) = quote.pop_quote("def!");
                // TODO: Typecheck...?
                definitions.insert(
                    name.clone(),
                    RailDef::from_quote(&name, description, commands),
                );
                (quote, definitions)
            })
        }),
        RailDef::on_state("alias", &format!("Consumes two commands, and binds the latter to the former. {}", DEFINITIONS_PRESERVED), &[QuoteOrCommand], &[], |state| {
            let conventions = state.conventions;
            state.update_stack_and_defs(|stack, mut definitions| {
                let (new_name, stack) = stack.pop();
                let (old_name, stack) = stack.pop();
                let (new_name, old_name) = if let (Some(new_name), Some(old_name)) = (get_command_name(&new_name), get_command_name(&old_name)) {
                    (new_name, old_name)
                } else {
                    rail_machine::log_warn(
                        conventions,
                        format!("Either {} or {} is not a command", old_name, new_name),
                    );
                    return (stack, definitions);
                };

                if let Some(old_definition) = definitions.get(&old_name) {
                    definitions.insert(
                        new_name.clone(),
                        old_definition.clone().rename(|_| new_name.clone())
                    );
                };

                (stack, definitions)
            })
        }),
        RailDef::on_state("=>", "Consumes a variable number of values, and binds them as one or more commands. Quotes are not expanded.", &[Unknown, QuoteOrCommand], &[], |state| {
            let child = state.child();
            state.update_stack_and_defs(|stack, mut definitions| {
                let (commands, stack) = stack.pop();
                let commands = commands.into_command_list();

                let stack = commands.into_iter().rev().fold(stack, |stack, command| {
                    let command_name = match command {
                        RailVal::Command(name) => name,
                        RailVal::DeferredCommand(name) => name,
                        _ => unreachable!()
                    };

                    let (val, stack) = stack.pop();

                    definitions.insert(
                        command_name.clone(),
                        RailDef::from_quote(&command_name, "FIXME: Undocumented", child.clone().push(val)),
                    );

                    stack
                });

                (stack, definitions)
            })
        }),
        RailDef::on_state("->", "Consumes a variable number of values, and binds them as one or more commands.", &[Unknown, QuoteOrCommand], &[], |state| {
            let child = state.child();
            state.update_stack_and_defs(|stack, mut definitions| {
                let (commands, stack) = stack.pop();
                let commands = commands.into_command_list();

                let stack = commands.into_iter().rev().fold(stack, |stack, command| {
                    let command_name = match command {
                        RailVal::Command(name) => name,
                        RailVal::DeferredCommand(name) => name,
                        _ => unreachable!()
                    };

                    let (val, stack) = stack.pop();

                    definitions.insert(
                        command_name.clone(),
                        RailDef::from_quote(&command_name, "FIXME: Undocumented", val.into_state(&child)),
                    );

                    stack
                });

                (stack, definitions)
            })
        }),
        RailDef::on_state("def?", "Consumes a quote or command, and produces true when it is defined, and false otherwise.", &[QuoteOrCommand], &[Boolean], |state| {
            let (name, state) = state.pop();
            let name = if let Some(name) = get_command_name(&name) {
                name
            } else {
                rail_machine::log_warn(
                    state.conventions,
                    format!("{} is not a string or command", name),
                );
                return state;
            };
            let is_def = state.definitions.contains_key(&name);
            state.push_bool(is_def)
        }),
        RailDef::on_state("describe", "Consumes a quoted command or command, and produces its description as a string.", &[QuoteOrCommand], &[String], |state| {
            let (name, state) = state.pop();
            let name = if let Some(name) = get_command_name(&name) {
                name
            } else {
                rail_machine::log_warn(
                    state.conventions,
                    format!("{} is not a string or command", name),
                );
                return state;
            };
            if state.definitions.contains_key(&name) {
                let description = state.definitions.get(&name).unwrap().description.clone();
                state.push_string(description)
            } else {
                state.push_string(format!("Command \"{}\" is unknown.", &name))
            }
        }),
    ]
}

fn do_it() -> impl Fn(RailState) -> RailState {
    |state| {
        let (command, state) = state.pop();

        match command {
            RailVal::Quote(quote) => quote.run_in_state(state),
            RailVal::Command(name) | RailVal::DeferredCommand(name) => {
                let action = state.get_def(&name).unwrap();
                action.clone().act(state.clone())
            }
            _ => {
                rail_machine::log_warn(
                    state.conventions,
                    format!("{} is not a quote or command", command),
                );
                state
            }
        }
    }
}

fn doin() -> impl Fn(RailState) -> RailState {
    |state| {
        let (commands, state) = state.pop();
        let commands = match &commands {
            RailVal::Command(name) | RailVal::DeferredCommand(name) => {
                state.child().push_command(name)
            }
            RailVal::Quote(commands) => commands.clone(),
            _ => {
                rail_machine::type_panic_msg("doin", "Command or Quote", commands);
                panic!()
            }
        };
        let (target, state) = state.pop();

        let targets = match target {
            RailVal::Quote(q) => q,
            scalar => state.child().push(scalar),
        };

        let substate = state.child().replace_stack(targets.stack);
        let substate = commands.run_in_state(substate);

        state.push_quote(substate)
    }
}

fn get_command_name(name: &RailVal) -> Option<std::string::String> {
    match name.clone() {
        RailVal::String(s) => Some(s),
        RailVal::Command(c) => Some(c),
        RailVal::DeferredCommand(c) => Some(c),
        RailVal::Quote(q) => {
            let (v, q) = q.pop();
            match (v, q.len()) {
                (RailVal::String(s), 0) => Some(s),
                (RailVal::Command(c), 0) => Some(c),
                (RailVal::DeferredCommand(c), 0) => Some(c),
                _ => None,
            }
        }
        _ => None,
    }
}
