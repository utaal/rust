use std::fmt::Debug;
use std::rc::Rc;

pub type RawSpan = Rc<dyn std::any::Any>;
#[derive(Clone)]
pub struct Span {
    pub raw_span: RawSpan,
    pub as_string: String,
}
pub type SpanOption = Rc<Option<Span>>;

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_tuple("Span").field(&self.as_string).finish()
    }
}

pub type TypeError = String;

#[derive(Debug)]
pub enum ValidityResult {
    Valid,
    Invalid(SpanOption),
    TypeError(TypeError),
}

pub type Ident = Rc<String>;

pub type Typ = Rc<TypX>;
pub type Typs = Rc<Vec<Typ>>;
#[derive(Debug)]
pub enum TypX {
    Bool,
    Int,
    Named(Ident),
}

#[derive(Clone, Debug)]
pub enum Constant {
    Bool(bool),
    Nat(Rc<String>),
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryOp {
    Not,
}

#[derive(Copy, Clone, Debug)]
pub enum BinaryOp {
    Implies,
    Eq,
    Le,
    Ge,
    Lt,
    Gt,
    EuclideanDiv,
    EuclideanMod,
}

#[derive(Copy, Clone, Debug)]
pub enum MultiOp {
    And,
    Or,
    Add,
    Sub,
    Mul,
    Distinct,
}

pub type Binder<A> = Rc<BinderX<A>>;
pub type Binders<A> = Rc<Vec<Binder<A>>>;
#[derive(Clone)]
pub struct BinderX<A: Clone> {
    pub name: Ident,
    pub a: A,
}

impl<A: Clone> BinderX<A> {
    pub fn map_a<B: Clone>(&self, f: impl FnOnce(&A) -> B) -> BinderX<B> {
        BinderX { name: self.name.clone(), a: f(&self.a) }
    }
}

impl<A: Clone + Debug> std::fmt::Debug for BinderX<A> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.name.fmt(fmt)?;
        fmt.write_str(" -> ")?;
        self.a.fmt(fmt)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Quant {
    Forall,
    Exists,
}

pub type Trigger = Exprs;
pub type Triggers = Rc<Vec<Trigger>>;

pub type Bind = Rc<BindX>;
#[derive(Clone, Debug)]
pub enum BindX {
    Let(Binders<Expr>),
    Quant(Quant, Binders<Typ>, Triggers),
}

pub type Expr = Rc<ExprX>;
pub type Exprs = Rc<Vec<Expr>>;
#[derive(Debug)]
pub enum ExprX {
    Const(Constant),
    Var(Ident),
    Apply(Ident, Exprs),
    Unary(UnaryOp, Expr),
    Binary(BinaryOp, Expr, Expr),
    Multi(MultiOp, Exprs),
    IfElse(Expr, Expr, Expr),
    Bind(Bind, Expr),
    LabeledAssertion(SpanOption, Expr),
}

pub type Stmt = Rc<StmtX>;
pub type Stmts = Rc<Vec<Stmt>>;
#[derive(Debug)]
pub enum StmtX {
    Assume(Expr),
    Assert(SpanOption, Expr),
    Assign(Ident, Expr),
    Block(Stmts),
}

pub type Field = Binder<Typ>;
pub type Fields = Binders<Typ>;
pub type Variant = Binder<Fields>;
pub type Variants = Binders<Fields>;
pub type Datatype = Binder<Variants>;
pub type Datatypes = Binders<Variants>;

pub type Decl = Rc<DeclX>;
pub type Decls = Rc<Vec<Decl>>;
#[derive(Debug)]
pub enum DeclX {
    Sort(Ident),
    Datatypes(Datatypes),
    Const(Ident, Typ),
    Fun(Ident, Typs, Typ),
    Var(Ident, Typ),
    Axiom(Expr),
}

pub type Query = Rc<QueryX>;
#[derive(Debug)]
pub struct QueryX {
    pub local: Decls,    // local declarations
    pub assertion: Stmt, // checked by SMT with global and local declarations
}

pub type Command = Rc<CommandX>;
pub type Commands = Rc<Vec<Command>>;
#[derive(Debug)]
pub enum CommandX {
    Push,                    // push space for temporary global declarations
    Pop,                     // pop temporary global declarations
    SetOption(Ident, Ident), // set-option option value (no colon on the option)
    Global(Decl),            // global declarations
    CheckValid(Query),       // SMT check-sat (reporting validity rather than satisfiability)
}
