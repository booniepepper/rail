use im::{HashMap, Vector};
use std::fmt::Display;
use std::sync::Arc;

use crate::tokens::Token;
use crate::v1::log;

#[derive(Clone)]
pub struct RunConventions<'a> {
    pub exe_name: &'a str,
    pub exe_version: &'a str,
    pub info_prefix: &'a str,
    pub warn_prefix: &'a str,
    pub error_prefix: &'a str,
    pub fatal_prefix: &'a str,
}

#[derive(Clone)]
pub enum RailError {
    UnknownCommand(String),
    StackUnderflow(RailState, String, Vec<RailType>),
    TypeMismatch(Vec<RailType>, Vec<RailVal>),
    CantEscape(Context),
}

impl std::fmt::Debug for RailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CantEscape(ctx) => write!(
                f,
                "Can't escape {}. This usually means there are too many closing brackets.",
                match ctx {
                    Context::Main => "main context",
                    Context::None => "contextless scope",
                    Context::Quotation { parent_state: _ } => "quotation",
                }
            ),
            Self::StackUnderflow(state, name, consumes) => write!(
                f,
                "Stack underflow. Stack had {} elements, but {} wanted {}",
                state.len(),
                name,
                consumes.len()
            ),
            Self::TypeMismatch(types, values) => {
                let values: Vec<RailType> = values.iter().map(|v| v.get_type()).collect();
                write!(f, "Type mismatch. Wanted {:?} but had {:?}", types, values)
            }
            Self::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
        }
    }
}

pub type RailRunResult = Result<RailState, (RailState, RailError)>;

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

    pub fn run_tokens(self, tokens: Vec<Token>) -> RailRunResult {
        tokens.iter().fold(Ok(self), |state, term| {
            state.and_then(|state| state.run_token(term.clone()))
        })
    }

    pub fn run_token(self, token: Token) -> RailRunResult {
        let res = match token {
            Token::None => self,
            Token::LeftBracket => self.deeper(),
            Token::RightBracket => return self.higher(),
            Token::String(s) => self.push_string(s),
            Token::Boolean(b) => self.push_bool(b),
            Token::I64(i) => self.push_i64(i),
            Token::F64(f) => self.push_f64(f),
            Token::DeferredTerm(term) => self.push_deferred_command(&term),
            Token::Term(term) => match (self.clone().get_def(&term), self.in_main()) {
                (Some(op), true) => {
                    return op.act(self);
                }
                (Some(op), false) => self.push_command(&op.name),
                (None, false) => self.push_command(&term),
                (None, true) => {
                    return Err((self, RailError::UnknownCommand(term.replace('\n', "\\n"))));
                }
            },
        };

        Ok(res)
    }

    pub fn run_val(self, value: RailVal, local_state: RailState) -> RailRunResult {
        match value {
            RailVal::Command(name) => {
                let state = self.clone();
                let cmd = state.get_def(&name).or_else(|| local_state.get_def(&name));

                match cmd {
                    None => Err((self, RailError::UnknownCommand(name))),
                    Some(cmd) => cmd.act(self),
                }
            }
            value => Ok(self.push(value)),
        }
    }

    pub fn run_in_state(self, other_state: RailState) -> RailRunResult {
        let values = self.stack.clone().values;
        values
            .into_iter()
            .fold(Ok(other_state), |state, value| match state {
                Ok(state) => state.run_val(value, self.child()),
                err => err,
            })
    }

    pub fn jailed_run_in_state(self, other_state: RailState) -> RailRunResult {
        let jailed = |state: RailState| other_state.clone().replace_stack(state.stack);
        self.run_in_state(other_state.clone())
            .map(jailed)
            .map_err(|(state, e)| (jailed(state), e))
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

    pub fn deeper(self) -> Self {
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

    pub fn higher(self) -> RailRunResult {
        match self.context.clone() {
            Context::Quotation { parent_state } => Ok(parent_state.push_quote(self)),
            context => Err((self, RailError::CantEscape(context))),
        }
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

    pub fn push_deferred_command(self, op_name: &str) -> Self {
        self.push(RailVal::DeferredCommand(op_name.to_owned()))
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
            _ => panic!("{}", log::type_panic_msg(context, "bool", value)),
        }
    }

    pub fn pop_i64(self, context: &str) -> (i64, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::I64(n) => (n, quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "i64", rail_val)),
        }
    }

    pub fn pop_f64(self, context: &str) -> (f64, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::F64(n) => (n, quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "f64", rail_val)),
        }
    }

    fn _pop_command(self, context: &str) -> (String, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Command(op) => (op, quote),
            RailVal::DeferredCommand(op) => (op, quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "command", rail_val)),
        }
    }

    pub fn pop_quote(self, context: &str) -> (RailState, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Quote(subquote) => (subquote, quote),
            // TODO: Can we coerce somehow?
            // RailVal::Stab(s) => (stab_to_quote(s), quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "quote", rail_val)),
        }
    }

    pub fn pop_stab(self, context: &str) -> (Stab, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Stab(s) => (s, quote),
            // TODO: Can we coerce somehow?
            // RailVal::Quote(q) => (quote_to_stab(q.stack), quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "string", rail_val)),
        }
    }

    pub fn pop_stab_entry(self, context: &str) -> (String, RailVal, Self) {
        let (original_entry, quote) = self.pop_quote(context);
        let (value, entry) = original_entry.clone().stack.pop();
        let (key, entry) = entry.pop_string(context);

        if !entry.is_empty() {
            panic!(
                "{}",
                log::type_panic_msg(context, "[ string a ]", RailVal::Quote(original_entry))
            );
        }

        (key, value, quote)
    }

    pub fn pop_string(self, context: &str) -> (String, Self) {
        let (value, quote) = self.pop();
        match value {
            RailVal::String(s) => (s, quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "string", rail_val)),
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

#[derive(Clone, Debug)]
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
    DeferredCommand(String),
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
            (DeferredCommand(a), DeferredCommand(b)) => a == b,
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
            RailVal::DeferredCommand(_) => RailType::Command,
            RailVal::Quote(_) => RailType::Quote,
            RailVal::String(_) => RailType::String,
            RailVal::Stab(_) => RailType::Stab,
        }
    }

    pub fn into_command_list(self) -> Vec<RailVal> {
        match &self {
            RailVal::Command(_) => vec![self],
            RailVal::DeferredCommand(_) => vec![self],
            RailVal::String(s) => vec![RailVal::Command(s.into())],
            RailVal::Quote(q) => q
                .clone()
                .stack
                .values
                .into_iter()
                .flat_map(|v| v.into_command_list())
                .collect(),
            _ => unimplemented!(),
        }
    }

    pub fn into_state(self, state: &RailState) -> RailState {
        match &self {
            RailVal::Quote(q) => q.clone(),
            _ => state.child().push(self),
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
            Command(cmd) => write!(fmt, "{}", cmd),
            DeferredCommand(cmd) => write!(fmt, "\\{}", cmd),
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
            _ => panic!("{}", log::type_panic_msg(context, "bool", value)),
        }
    }

    pub fn pop_i64(self, context: &str) -> (i64, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::I64(n) => (n, quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "i64", rail_val)),
        }
    }

    pub fn pop_f64(self, context: &str) -> (f64, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::F64(n) => (n, quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "f64", rail_val)),
        }
    }

    fn _pop_command(self, context: &str) -> (String, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Command(op) => (op, quote),
            RailVal::DeferredCommand(op) => (op, quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "command", rail_val)),
        }
    }

    pub fn pop_quote(self, context: &str) -> (RailState, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Quote(subquote) => (subquote, quote),
            // TODO: Can we coerce somehow?
            // RailVal::Stab(s) => (stab_to_quote(s), quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "quote", rail_val)),
        }
    }

    pub fn pop_stab(self, context: &str) -> (Stab, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::Stab(s) => (s, quote),
            // TODO: Can we coerce somehow?
            // RailVal::Quote(q) => (quote_to_stab(q.values), quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "string", rail_val)),
        }
    }

    pub fn pop_stab_entry(self, context: &str) -> (String, RailVal, Stack) {
        let (original_entry, quote) = self.pop_quote(context);
        let (value, entry) = original_entry.clone().stack.pop();
        let (key, entry) = entry.pop_string(context);

        if !entry.is_empty() {
            panic!(
                "{}",
                log::type_panic_msg(context, "[ string a ]", RailVal::Quote(original_entry))
            );
        }

        (key, value, quote)
    }

    pub fn pop_string(self, context: &str) -> (String, Stack) {
        let (value, quote) = self.pop();
        match value {
            RailVal::String(s) => (s, quote),
            rail_val => panic!("{}", log::type_panic_msg(context, "string", rail_val)),
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
    pub description: String,
    consumes: &'a [RailType],
    produces: &'a [RailType],
    action: RailAction<'a>,
}

#[derive(Clone)]
pub enum RailAction<'a> {
    Builtin(Arc<dyn Fn(RailState) -> RailRunResult + 'a>),
    BuiltinSafe(Arc<dyn Fn(RailState) -> RailState + 'a>),
    Quotation(RailState),
}

impl<'a> RailDef<'a> {
    pub fn on_state<F>(
        name: &str,
        description: &str,
        consumes: &'a [RailType],
        produces: &'a [RailType],
        state_action: F,
    ) -> RailDef<'a>
    where
        F: Fn(RailState) -> RailRunResult + 'a,
    {
        RailDef {
            name: name.to_string(),
            description: description.to_string(),
            consumes,
            produces,
            action: RailAction::Builtin(Arc::new(state_action)),
        }
    }

    // TODO: Make this fn stop existing, or at least being so widely used. Like `on_state` but does not return RailRunResult type.
    pub fn on_state_noerr<F>(
        name: &str,
        description: &str,
        consumes: &'a [RailType],
        produces: &'a [RailType],
        state_action: F,
    ) -> RailDef<'a>
    where
        F: Fn(RailState) -> RailState + 'a,
    {
        RailDef {
            name: name.to_string(),
            description: description.to_string(),
            consumes,
            produces,
            action: RailAction::BuiltinSafe(Arc::new(state_action)),
        }
    }

    pub fn on_jailed_state<F>(
        name: &str,
        description: &str,
        consumes: &'a [RailType],
        produces: &'a [RailType],
        state_action: F,
    ) -> RailDef<'a>
    where
        F: Fn(RailState) -> RailRunResult + 'a,
    {
        RailDef {
            name: name.to_string(),
            description: description.to_string(),
            consumes,
            produces,
            action: RailAction::Builtin(Arc::new(move |state| {
                let definitions = state.definitions.clone();
                let substate = state_action(state)?;
                Ok(substate.replace_definitions(definitions))
            })),
        }
    }

    pub fn contextless<F>(
        name: &str,
        description: &str,
        consumes: &'a [RailType],
        produces: &'a [RailType],
        contextless_action: F,
    ) -> RailDef<'a>
    where
        F: Fn() + 'a,
    {
        RailDef::on_state(name, description, consumes, produces, move |state| {
            contextless_action();
            Ok(state)
        })
    }

    pub fn from_quote(name: &str, description: &str, quote: RailState) -> RailDef<'a> {
        // TODO: Infer quote effects
        RailDef {
            name: name.to_string(),
            description: description.to_string(),
            consumes: &[],
            produces: &[],
            action: RailAction::Quotation(quote),
        }
    }

    pub fn act(self, state: RailState) -> RailRunResult {
        if state.stack.len() < self.consumes.len() {
            // TODO: At some point will want source context here like line/column number.
            return Err((
                state.clone(),
                RailError::StackUnderflow(state, self.name, self.consumes.to_vec()),
            ));
        }

        // TODO: Type checks?

        match self.action {
            RailAction::Builtin(action) => action(state),
            RailAction::BuiltinSafe(action) => Ok(action(state)),
            RailAction::Quotation(quote) => quote.run_in_state(state),
        }
    }

    pub fn rename<F>(self, f: F) -> RailDef<'a>
    where
        F: Fn(String) -> String,
    {
        RailDef {
            name: f(self.name),
            description: self.description,
            consumes: self.consumes,
            produces: self.produces,
            action: self.action,
        }
    }

    pub fn redescribe<F>(self, f: F) -> RailDef<'a>
    where
        F: Fn(String) -> String,
    {
        RailDef {
            name: self.name,
            description: f(self.description),
            consumes: self.consumes,
            produces: self.produces,
            action: self.action,
        }
    }
}
