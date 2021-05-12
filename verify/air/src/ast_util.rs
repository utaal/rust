use crate::ast::{
    BinaryOp, BindX, Binder, BinderX, Constant, Expr, ExprX, Ident, MultiOp, Quant, Span, Trigger,
    Typ, TypX, UnaryOp,
};
use std::fmt::Debug;
use std::rc::Rc;

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_tuple("Span").field(&self.as_string).finish()
    }
}

impl Debug for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Constant::Bool(b) => write!(f, "{}", b),
            Constant::Nat(n) => write!(f, "{}", n),
        }
    }
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

pub fn str_ident(x: &str) -> Ident {
    Rc::new(x.to_string())
}

pub fn ident_var(x: &Ident) -> Expr {
    Rc::new(ExprX::Var(x.clone()))
}

pub fn string_var(x: &String) -> Expr {
    Rc::new(ExprX::Var(Rc::new(x.clone())))
}

pub fn str_var(x: &str) -> Expr {
    Rc::new(ExprX::Var(Rc::new(x.to_string())))
}

pub fn ident_apply(x: &Ident, args: &Vec<Expr>) -> Expr {
    Rc::new(ExprX::Apply(x.clone(), Rc::new(args.clone())))
}

pub fn string_apply(x: &String, args: &Vec<Expr>) -> Expr {
    Rc::new(ExprX::Apply(Rc::new(x.clone()), Rc::new(args.clone())))
}

pub fn str_apply(x: &str, args: &Vec<Expr>) -> Expr {
    Rc::new(ExprX::Apply(Rc::new(x.to_string()), Rc::new(args.clone())))
}

pub fn int_typ() -> Typ {
    Rc::new(TypX::Int)
}

pub fn bool_typ() -> Typ {
    Rc::new(TypX::Bool)
}

pub fn ident_typ(x: &Ident) -> Typ {
    Rc::new(TypX::Named(x.clone()))
}

pub fn string_typ(x: &String) -> Typ {
    Rc::new(TypX::Named(Rc::new(x.clone())))
}

pub fn str_typ(x: &str) -> Typ {
    Rc::new(TypX::Named(Rc::new(x.to_string())))
}

pub fn ident_binder<A: Clone>(x: &Ident, a: &A) -> Binder<A> {
    Rc::new(BinderX { name: x.clone(), a: a.clone() })
}

pub fn mk_quantifier(
    quant: Quant,
    binders: &Vec<Binder<Typ>>,
    triggers: &Vec<Trigger>,
    body: &Expr,
) -> Expr {
    Rc::new(ExprX::Bind(
        Rc::new(BindX::Quant(quant, Rc::new(binders.clone()), Rc::new(triggers.clone()))),
        body.clone(),
    ))
}

pub fn mk_forall(binders: &Vec<Binder<Typ>>, triggers: &Vec<Trigger>, body: &Expr) -> Expr {
    mk_quantifier(Quant::Forall, binders, triggers, body)
}

pub fn mk_exists(binders: &Vec<Binder<Typ>>, triggers: &Vec<Trigger>, body: &Expr) -> Expr {
    mk_quantifier(Quant::Exists, binders, triggers, body)
}

pub fn mk_true() -> Expr {
    Rc::new(ExprX::Const(Constant::Bool(true)))
}

pub fn mk_false() -> Expr {
    Rc::new(ExprX::Const(Constant::Bool(false)))
}

pub fn mk_and(exprs: &Vec<Expr>) -> Expr {
    if exprs.iter().any(|expr| matches!(**expr, ExprX::Const(Constant::Bool(false)))) {
        return mk_false();
    }
    let exprs: Vec<Expr> = exprs
        .iter()
        .filter(|expr| !matches!(***expr, ExprX::Const(Constant::Bool(true))))
        .cloned()
        .collect();
    if exprs.len() == 0 {
        mk_true()
    } else if exprs.len() == 1 {
        exprs[0].clone()
    } else {
        Rc::new(ExprX::Multi(MultiOp::And, Rc::new(exprs)))
    }
}

pub fn mk_or(exprs: &Vec<Expr>) -> Expr {
    if exprs.iter().any(|expr| matches!(**expr, ExprX::Const(Constant::Bool(true)))) {
        return mk_true();
    }
    let exprs: Vec<Expr> = exprs
        .iter()
        .filter(|expr| !matches!(***expr, ExprX::Const(Constant::Bool(false))))
        .cloned()
        .collect();
    if exprs.len() == 0 {
        mk_false()
    } else if exprs.len() == 1 {
        exprs[0].clone()
    } else {
        Rc::new(ExprX::Multi(MultiOp::Or, Rc::new(exprs)))
    }
}

pub fn mk_not(e1: &Expr) -> Expr {
    match &**e1 {
        ExprX::Const(Constant::Bool(false)) => mk_true(),
        ExprX::Const(Constant::Bool(true)) => mk_false(),
        ExprX::Unary(UnaryOp::Not, e) => e.clone(),
        _ => Rc::new(ExprX::Unary(UnaryOp::Not, e1.clone())),
    }
}

pub fn mk_implies(e1: &Expr, e2: &Expr) -> Expr {
    match (&**e1, &**e2) {
        (ExprX::Const(Constant::Bool(false)), _) => mk_true(),
        (ExprX::Const(Constant::Bool(true)), _) => e2.clone(),
        (_, ExprX::Const(Constant::Bool(false))) => mk_not(e1),
        (_, ExprX::Const(Constant::Bool(true))) => mk_true(),
        _ => Rc::new(ExprX::Binary(BinaryOp::Implies, e1.clone(), e2.clone())),
    }
}

pub fn mk_ite(e1: &Expr, e2: &Expr, e3: &Expr) -> Expr {
    match (&**e1, &**e2, &**e3) {
        (ExprX::Const(Constant::Bool(true)), _, _) => e2.clone(),
        (ExprX::Const(Constant::Bool(false)), _, _) => e3.clone(),
        (_, _, ExprX::Const(Constant::Bool(true))) => mk_implies(e1, e2),
        (_, _, ExprX::Const(Constant::Bool(false))) => mk_and(&vec![e1.clone(), e2.clone()]),
        (_, ExprX::Const(Constant::Bool(true)), _) => mk_implies(&mk_not(e1), e3),
        (_, ExprX::Const(Constant::Bool(false)), _) => mk_and(&vec![mk_not(e1), e3.clone()]),
        _ => Rc::new(ExprX::IfElse(e1.clone(), e2.clone(), e3.clone())),
    }
}

pub fn mk_eq(e1: &Expr, e2: &Expr) -> Expr {
    Rc::new(ExprX::Binary(BinaryOp::Eq, e1.clone(), e2.clone()))
}
