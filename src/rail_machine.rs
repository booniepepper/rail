use colored::Colorize;

use im::{HashMap, Vector};
use std::fmt::Display;
use std::sync::Arc;

#[derive(Clone)]
pub struct RunConventions<'a> {
    pub exe_name: &'a str,
    pub exe_version: &'a str,
    pub warn_prefix: &'a str,
    pub fatal_prefix: &'a str,
}

#[derive(Clone)]
pub struct RailState {
    // TODO: Provide update functions and make these private
    pub stack: Stack,
    pub definitions: Dictionary,
    // TODO: Save parents at time of definition and at runtime
    pub context: Context,
    pub conventions: &'static RunConventions<'static>,
}

impl RailState {
    pub fn new(
        context: Context,
        definitions: Dictionary,
        conventions: &'static RunConventions,
    ) -> RailState {
        let stack = Stack::default();
        RailState {
            stack,
            definitions,
            context,
            conventions,
        }
    }

    pub fn new_main(definitions: Dictionary, conventions: &'static RunConventions) -> RailState {
        RailState::new(Context::Main, definitions, conventions)
    }

    pub fn in_main(&self) -> bool {
        matches!(self.context, Context::Main)
    }

    pub fn get_def(&self, name: &str) -> Option<RailDef> {
        self.definitions.get(name).cloned()
    }

    pub fn child(&self) -> Self {
        RailState {
            stack: Stack::default(),
            definitions: self.definitions.clone(),
            context: Context::None,
            conventions: self.conventions,
        }
    }

    pub fn run_tokens(self, tokens: Vec<String>) -> RailState {
        tokens.iter().fold(self, |state, term| state.run_term(term))
    }

    pub fn run_term<S>(self, term: S) -> RailState
    where
        S: Into<String>,
    {
        let term: String = term.into();

        // Quotations
        if term == "[" {
            self.deeper()
        } else if term == "]" {
            self.higher()
        }
        // Defined operations
        else if let Some(op) = self.clone().get_def(&term) {
            if self.in_main() {
                let mut op = op;
                op.act(self)
            } else {
                self.push_command(&op.name)
            }
        }
        // Strings
        else if term.starts_with('"') && term.ends_with('"') {
            let term = term.strip_prefix('"').unwrap().strip_suffix('"').unwrap();
            self.push_str(term)
        }
        // Integers
        else if let Ok(i) = term.parse::<i64>() {
            self.push_i64(i)
        }
        // Floating point numbers
        else if let Ok(n) = term.parse::<f64>() {
            self.push_f64(n)
        }
        // Unknown
        else if !self.in_main() {
            // We optimistically expect this may be a not-yet-defined term. This
            // gives a way to do recursive definitions.
            self.push_command(&term)
        } else {
            // TODO: Use a logging library? Log levels? Exit in a strict mode?
            // TODO: Have/get details on filename/source, line number, character number
            let term = term.replace('\n', "\\n");
            derail_for_unknown_command(&term, self.conventions);
        }
    }

    pub fn run_val(&self, value: RailVal, local_state: RailState) -> RailState {
        match value {
            RailVal::Command(name) => {
                let mut cmd = self
                    .get_def(&name)
                    .or_else(|| local_state.get_def(&name))
                    .unwrap_or_else(|| derail_for_unknown_command(&name, self.conventions));
                cmd.act(self.clone())
            }
            value => self.clone().push(value),
        }
    }

    pub fn run_in_state(self, other_state: RailState) -> RailState {
        let values = self.stack.clone().values;
        values.into_iter().fold(other_state, |state, value| {
            state.run_val(value, self.child())
        })
    }

    pub fn jailed_run_in_state(self, other_state: RailState) -> RailState {
        let after_run = self.run_in_state(other_state.clone());
        other_state.replace_stack(after_run.stack)
    }

    pub fn update_stack(self, update: impl Fn(Stack) -> Stack) -> RailState {
        RailState {
            stack: update(self.stack),
            definitions: self.definitions,
            context: self.context,
            conventions: self.conventions,
        }
    }

    pub fn update_stack_and_defs(
        self,
        update: impl Fn(Stack, Dictionary) -> (Stack, Dictionary),
    ) -> RailState {
        let (stack, definitions) = update(self.stack, self.definitions);
        RailState {
            stack,
            definitions,
            context: self.context,
            conventions: self.conventions,
        }
    }

    pub fn replace_stack(self, stack: Stack) -> RailState {
        RailState {
            stack,
            definitions: self.definitions,
            context: self.context,
            conventions: self.conventions,
        }
    }

    pub fn replace_definitions(self, definitions: Dictionary) -> RailState {
        RailState {
            stack: self.stack,
            definitions,
            context: self.context,
            conventions: self.conventions,
        }
    }

    pub fn replace_context(self, context: Context) -> RailState {
        RailState {
            stack: self.stack,
            definitions: self.definitions,
            context,
            conventions: self.conventions,
        }
    }

    pub fn deeper(self) -> RailState {
        let conventions = self.conventions;
        RailState {
            stack: Stack::default(),
            definitions: self.definitions.clone(),
            context: Context::Quotation {
                parent_state: Box::new(self),
            },
            conventions,
        }
    }

    pub fn higher(self) -> RailState {
        let parent_state = match self.context.clone() {
            Context::Quotation { parent_state } => *parent_state,
            Context::Main => panic!("Can't escape main"),
            Context::None => panic!("Can't escape"),
        };

        parent_state.push_quote(self)
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn reverse(self) -> Self {
        self.update_stack(|stack| stack.reverse())
    }

    pub fn push(self, term: RailVal) -> Self {
        self.update_stack(|stack| stack.push(term.clone()))
    }

    pub fn push_bool(self, b: bool) -> Self {
        self.push(RailVal::Boolean(b))
    }

    pub fn push_i64(self, i: i64) -> Self {
        self.push(RailVal::I64(i))
    }

    pub fn push_f64(self, n: f64) -> Self {
        self.push(RailVal::F64(n))
    }

    pub fn push_command(self, op_name: &str) -> Self {
        self.push(RailVal::Command(op_name.to_owned()))
    }

    pub fn push_quote(self, quote: RailState) -> Self {
        self.push(RailVal::Quote(quote))
    }

    pub fn push_stab(self, st: Stab) -> Self {
        self.push(RailVal::Stab(st))
    }

    pub fn push_string(self, s: String) -> Self {
        self.push(RailVal::String(s))
    }

    pub fn push_str(self, s: &str) -> Self {
        self.push(RailVal::String(s.to_owned()))
    }

    pub fn pop(self) -> (RailVal, Self) {
        let (value, stack) = self.stack.clone().pop();
        (value, self.replace_stack(stack))
    }

    pub fn pop_bool(self, context: &str) -> (bool, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Boolean(b) => (b, quote),
            _ => panic!("{}", type_panic_msg(context, "bool", value)),
        }
    }

    pub fn pop_i64(self, context: &str) -> (i64, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::I64(n) => (n, quote),
            rail_val => panic!("{}", type_panic_msg(context, "i64", rail_val)),
        }
    }

    pub fn pop_f64(self, context: &str) -> (f64, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::F64(n) => (n, quote),
            rail_val => panic!("{}", type_panic_msg(context, "f64", rail_val)),
        }
    }

    fn _pop_command(self, context: &str) -> (String, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Command(op) => (op, quote),
            rail_val => panic!("{}", type_panic_msg(context, "command", rail_val)),
        }
    }

    pub fn pop_quote(self, context: &str) -> (RailState, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Quote(subquote) => (subquote, quote),
            // TODO: Can we coerce somehow?
            // RailVal::Stab(s) => (stab_to_quote(s), quote),
            rail_val => panic!("{}", type_panic_msg(context, "quote", rail_val)),
        }
    }

    pub fn pop_stab(self, context: &str) -> (Stab, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Stab(s) => (s, quote),
            // TODO: Can we coerce somehow?
            // RailVal::Quote(q) => (quote_to_stab(q.stack), quote),
            rail_val => panic!("{}", type_panic_msg(context, "string", rail_val)),
        }
    }

    pub fn pop_stab_entry(self, context: &str) -> (String, RailVal, Self) {
        let (original_entry, quote) = self.pop_quote(context);
        let (value, entry) = original_entry.clone().stack.pop();
        let (key, entry) = entry.pop_string(context);

        if !entry.is_empty() {
            panic!(
                "{}",
                type_panic_msg(context, "[ string a ]", RailVal::Quote(original_entry))
            );
        }

        (key, value, quote)
    }

    pub fn pop_string(self, context: &str) -> (String, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::String(s) => (s, quote),
            rail_val => panic!("{}", type_panic_msg(context, "string", rail_val)),
        }
    }

    pub fn enqueue(self, value: RailVal) -> Self {
        let stack = self.stack.clone().enqueue(value);
        self.replace_stack(stack)
    }

    pub fn dequeue(self) -> (RailVal, Self) {
        let (value, stack) = self.stack.clone().dequeue();
        (value, self.replace_stack(stack))
    }
}

#[derive(Clone)]
pub enum Context {
    Main,
    Quotation { parent_state: Box<RailState> },
    None,
}

pub enum RailType {
    A,
    B,
    C,
    /// Zero or many unknown types.
    Unknown,
    Boolean,
    Number,
    I64,
    F64,
    Command,
    // TODO: have quotes with typed contents
    // Examples: Quote<String...> for split
    //           Quote<String, Unknown> for stab entries
    Quote,
    QuoteOrCommand,
    QuoteOrString,
    String,
    Stab,
}

impl Display for RailType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RailType::*;
        let my_type = match self {
            A => "a",
            B => "b",
            C => "c",
            Unknown => "...",
            Boolean => "bool",
            Number => "num",
            I64 => "i64",
            F64 => "f64",
            Command => "command",
            Quote => "quote",
            QuoteOrCommand => "quote|command",
            QuoteOrString => "quote|string",
            String => "string",
            Stab => "stab",
        };

        write!(fmt, "{}", my_type)
    }
}

#[derive(Clone)]
pub enum RailVal {
    Boolean(bool),
    // TODO: Make a "Numeric" typeclass. (And floating-point/rational numbers)
    I64(i64),
    F64(f64),
    Command(String),
    Quote(RailState),
    String(String),
    Stab(Stab),
}

impl PartialEq for RailVal {
    fn eq(&self, other: &Self) -> bool {
        use RailVal::*;
        match (self, other) {
            (Boolean(a), Boolean(b)) => a == b,
            (I64(a), I64(b)) => a == b,
            (I64(a), F64(b)) => *a as f64 == *b,
            (F64(a), I64(b)) => *a == *b as f64,
            (F64(a), F64(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Command(a), Command(b)) => a == b,
            // TODO: For quotes, what about differing dictionaries? For simple lists they don't matter, for closures they do.
            (Quote(a), Quote(b)) => a.stack == b.stack,
            (Stab(a), Stab(b)) => a == b,
            _ => false,
        }
    }
}

impl RailVal {
    pub fn type_name(&self) -> String {
        self.get_type().to_string()
    }

    fn get_type(&self) -> RailType {
        match self {
            RailVal::Boolean(_) => RailType::Boolean,
            RailVal::I64(_) => RailType::I64,
            RailVal::F64(_) => RailType::F64,
            RailVal::Command(_) => RailType::Command,
            RailVal::Quote(_) => RailType::Quote,
            RailVal::String(_) => RailType::String,
            RailVal::Stab(_) => RailType::Stab,
        }
    }
}

impl std::fmt::Display for RailVal {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use RailVal::*;
        match self {
            Boolean(b) => write!(fmt, "{}", if *b { "true" } else { "false" }),
            I64(n) => write!(fmt, "{}", n),
            F64(n) => write!(fmt, "{}", n),
            Command(o) => write!(fmt, "{}", o),
            Quote(q) => write!(fmt, "{}", q.stack),
            String(s) => write!(fmt, "\"{}\"", s.replace('\n', "\\n")),
            Stab(t) => {
                write!(fmt, "[ ").unwrap();

                for (k, v) in t.iter() {
                    write!(fmt, "[ \"{}\" {} ] ", k, v).unwrap();
                }

                write!(fmt, "]")
            }
        }
    }
}

#[derive(Clone)]
pub struct Stack {
    pub values: Vector<RailVal>,
}

impl PartialEq for Stack {
    // FIXME: Not equal if inequal shadows (same name, diff binding) exist in the values
    fn eq(&self, other: &Self) -> bool {
        self.values
            .clone()
            .into_iter()
            .zip(other.values.clone())
            .all(|(a, b)| a == b)
    }
}

impl Stack {
    pub fn new(values: Vector<RailVal>) -> Self {
        Stack { values }
    }

    pub fn of(value: RailVal) -> Self {
        let mut values = Vector::default();
        values.push_back(value);
        Stack { values }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn reverse(&self) -> Stack {
        let values = self.values.iter().rev().cloned().collect();
        Stack::new(values)
    }

    pub fn push(mut self, term: RailVal) -> Stack {
        self.values.push_back(term);
        self
    }

    pub fn pop(mut self) -> (RailVal, Stack) {
        let term = self.values.pop_back().unwrap();
        (term, self)
    }

    pub fn pop_bool(self, context: &str) -> (bool, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Boolean(b) => (b, quote),
            _ => panic!("{}", type_panic_msg(context, "bool", value)),
        }
    }

    pub fn pop_i64(self, context: &str) -> (i64, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::I64(n) => (n, quote),
            rail_val => panic!("{}", type_panic_msg(context, "i64", rail_val)),
        }
    }

    pub fn pop_f64(self, context: &str) -> (f64, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::F64(n) => (n, quote),
            rail_val => panic!("{}", type_panic_msg(context, "f64", rail_val)),
        }
    }

    fn _pop_command(self, context: &str) -> (String, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Command(op) => (op, quote),
            rail_val => panic!("{}", type_panic_msg(context, "command", rail_val)),
        }
    }

    pub fn pop_quote(self, context: &str) -> (RailState, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Quote(subquote) => (subquote, quote),
            // TODO: Can we coerce somehow?
            // RailVal::Stab(s) => (stab_to_quote(s), quote),
            rail_val => panic!("{}", type_panic_msg(context, "quote", rail_val)),
        }
    }

    pub fn pop_stab(self, context: &str) -> (Stab, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Stab(s) => (s, quote),
            // TODO: Can we coerce somehow?
            // RailVal::Quote(q) => (quote_to_stab(q.values), quote),
            rail_val => panic!("{}", type_panic_msg(context, "string", rail_val)),
        }
    }

    pub fn pop_stab_entry(self, context: &str) -> (String, RailVal, Stack) {
        let (original_entry, quote) = self.pop_quote(context);
        let (value, entry) = original_entry.clone().stack.pop();
        let (key, entry) = entry.pop_string(context);

        if !entry.is_empty() {
            panic!(
                "{}",
                type_panic_msg(context, "[ string a ]", RailVal::Quote(original_entry))
            );
        }

        (key, value, quote)
    }

    pub fn pop_string(self, context: &str) -> (String, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::String(s) => (s, quote),
            rail_val => panic!("{}", type_panic_msg(context, "string", rail_val)),
        }
    }

    pub fn enqueue(mut self, value: RailVal) -> Stack {
        self.values.push_front(value);
        self
    }

    pub fn dequeue(mut self) -> (RailVal, Stack) {
        let value = self.values.pop_front().unwrap();
        (value, self)
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new(Vector::default())
    }
}

impl std::fmt::Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "[ ").unwrap();

        for term in &self.values {
            write!(f, "{} ", term).unwrap();
        }

        write!(f, "]").unwrap();

        Ok(())
    }
}

pub type Dictionary = HashMap<String, RailDef<'static>>;

pub fn dictionary_of<Entries>(entries: Entries) -> Dictionary
where
    Entries: IntoIterator<Item = RailDef<'static>>,
{
    let entries = entries.into_iter().map(|def| (def.name.clone(), def));
    HashMap::from_iter(entries)
}

pub type Stab = HashMap<String, RailVal>;

pub fn new_stab() -> Stab {
    HashMap::new()
}

#[derive(Clone)]
pub struct RailDef<'a> {
    pub name: String,
    consumes: &'a [RailType],
    produces: &'a [RailType],
    action: RailAction<'a>,
}

#[derive(Clone)]
pub enum RailAction<'a> {
    Builtin(Arc<dyn Fn(RailState) -> RailState + 'a>),
    Quotation(RailState),
}

impl <'a> RailDef<'a> {
    pub fn on_state<F>(
        name: &str,
        consumes: &'a [RailType],
        produces: &'a [RailType],
        state_action: F,
    ) -> RailDef<'a>
    where
        F: Fn(RailState) -> RailState + 'a,
    {
        RailDef {
            name: name.to_string(),
            consumes,
            produces,
            action: RailAction::Builtin(Arc::new(state_action)),
        }
    }

    pub fn on_jailed_state<F>(
        name: &str,
        consumes: &'a [RailType],
        produces: &'a [RailType],
        state_action: F,
    ) -> RailDef<'a>
    where
        F: Fn(RailState) -> RailState + 'a,
    {
        RailDef {
            name: name.to_string(),
            consumes,
            produces,
            action: RailAction::Builtin(Arc::new(move |state| {
                let definitions = state.definitions.clone();
                let substate = state_action(state);
                substate.replace_definitions(definitions)
            })),
        }
    }

    pub fn contextless<F>(
        name: &str,
        consumes: &'a [RailType],
        produces: &'a [RailType],
        contextless_action: F,
    ) -> RailDef<'a>
    where
        F: Fn() + 'a,
    {
        RailDef::on_state(name, consumes, produces, move |state| {
            contextless_action();
            state
        })
    }

    pub fn from_quote(name: &str, quote: RailState) -> RailDef<'a> {
        // TODO: Infer quote effects
        RailDef {
            name: name.to_string(),
            consumes: &[],
            produces: &[],
            action: RailAction::Quotation(quote),
        }
    }

    pub fn act(&mut self, state: RailState) -> RailState {
        if state.stack.len() < self.consumes.len() {
            // TODO: At some point will want source context here like line/column number.
            log_warn(
                state.conventions,
                format!(
                    "Underflow for \"{}\" (takes: {}, gives: {}). State: {}",
                    self.name,
                    self.display_consumes(),
                    self.display_produces(),
                    state.stack
                ),
            );
            return state;
        }

        // TODO: Type checks?

        match &self.action {
            RailAction::Builtin(action) => action(state),
            RailAction::Quotation(quote) => quote.clone().run_in_state(state),
        }
    }

    pub fn rename<F>(self, f: F) -> RailDef<'a>
    where
        F: Fn(String) -> String,
    {
        RailDef {
            name: f(self.name),
            consumes: self.consumes,
            produces: self.produces,
            action: self.action,
        }
    }

    fn display_consumes(&self) -> String {
        self.consumes
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn display_produces(&self) -> String {
        self.produces
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// The following are all handling for errors, warnings, and panics.
// TODO: Update places these are referenced to return Result.

pub fn derail_for_unknown_command(name: &str, conv: &RunConventions) -> ! {
    log_fatal(conv, format!("Unknown command '{}'", name));
    std::process::exit(1)
}

pub fn type_panic_msg(context: &str, expected: &str, actual: RailVal) -> String {
    format!(
        "[Context: {}] Wanted {}, but got {}",
        context, expected, actual
    )
}

pub fn log_warn(conv: &RunConventions, thing: impl Display) {
    let msg = format!("{}: {}", conv.warn_prefix, thing).dimmed().red();
    eprintln!("{}", msg);
}

pub fn log_fatal(conv: &RunConventions, thing: impl Display) {
    let msg = format!("{}: {}", conv.fatal_prefix, thing).dimmed().red();
    eprintln!("{}", msg);
}
