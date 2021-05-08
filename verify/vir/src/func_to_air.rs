use crate::ast::{Function, Ident, Idents, Mode, ParamX, Params, VirErr};
use crate::context::Ctx;
use crate::def::{
    prefix_ensures, prefix_fuel_id, prefix_recursive, prefix_requires, suffix_global_id,
    suffix_local_id, suffix_typ_param_id, Spanned, FUEL_BOOL, FUEL_BOOL_DEFAULT, FUEL_LOCAL,
    FUEL_NAT, FUEL_NAT_DEFAULT, FUEL_PARAM, FUEL_TYPE, SUCC,
};
use crate::sst_to_air::{exp_to_expr, typ_invariant, typ_to_air};
use crate::util::{vec_map, vec_map_result};
use air::ast::{
    BinaryOp, Bind, BindX, Command, CommandX, Commands, DeclX, Expr, ExprX, MultiOp, Quant, Span,
    Trigger, Triggers,
};
use air::ast_util::{
    bool_typ, ident_apply, ident_binder, ident_var, mk_and, mk_eq, mk_exists, mk_implies,
    str_apply, str_ident, str_typ, str_var, string_apply,
};
use std::rc::Rc;

fn func_bind(typ_params: &Idents, params: &Params, trig_expr: &Expr, add_fuel: bool) -> Bind {
    let mut binders: Vec<air::ast::Binder<air::ast::Typ>> = Vec::new();
    for typ_param in typ_params.iter() {
        binders.push(ident_binder(&suffix_typ_param_id(&typ_param), &str_typ(crate::def::TYPE)));
    }
    for param in params.iter() {
        binders
            .push(ident_binder(&suffix_local_id(&param.x.name.clone()), &typ_to_air(&param.x.typ)));
    }
    if add_fuel {
        binders.push(ident_binder(&str_ident(FUEL_LOCAL), &str_typ(FUEL_TYPE)));
    }
    let trigger: Trigger = Rc::new(vec![trig_expr.clone()]);
    let triggers: Triggers = Rc::new(vec![trigger]);
    Rc::new(BindX::Quant(Quant::Forall, Rc::new(binders), triggers))
}

fn func_def_args(typ_params: &Idents, params: &Params) -> Vec<Expr> {
    let mut f_args: Vec<Expr> = Vec::new();
    for typ_param in typ_params.iter() {
        f_args.push(ident_var(&suffix_typ_param_id(&typ_param)));
    }
    for param in params.iter() {
        f_args.push(ident_var(&suffix_local_id(&param.x.name.clone())));
    }
    f_args
}

// (forall (...) (= (f ...) body))
fn func_def_quant(
    name: &Ident,
    typ_params: &Idents,
    params: &Params,
    body: Expr,
) -> Result<Expr, VirErr> {
    let f_args = func_def_args(typ_params, params);
    let f_app = string_apply(name, &Rc::new(f_args));
    let f_eq = Rc::new(ExprX::Binary(BinaryOp::Eq, f_app.clone(), body));
    Ok(Rc::new(ExprX::Bind(func_bind(typ_params, params, &f_app, false), f_eq)))
}

fn func_body_to_air(
    ctx: &Ctx,
    commands: &mut Vec<Command>,
    function: &Function,
    body: &crate::ast::Expr,
) -> Result<(), VirErr> {
    let id_fuel = prefix_fuel_id(&function.x.name);

    // ast --> sst
    let body_exp = crate::ast_to_sst::expr_to_exp(&ctx, &body)?;

    // Check termination
    let (is_recursive, termination_commands, body_exp) =
        crate::recursion::check_termination_exp(ctx, function, &body_exp)?;
    commands.extend(termination_commands.iter().cloned());

    // non-recursive:
    //   (axiom (fuel_bool_default fuel%f))
    // recursive:
    //   (axiom (exists ((fuel Fuel)) (= (fuel_nat_default fuel%f) (succ fuel))))
    if function.x.fuel > 0 {
        let axiom_expr = if !is_recursive {
            str_apply(&FUEL_BOOL_DEFAULT, &vec![ident_var(&id_fuel)])
        } else {
            let mut default_fuel = str_var(FUEL_PARAM);
            for _ in 0..function.x.fuel {
                default_fuel = str_apply(SUCC, &vec![default_fuel]);
            }
            let fuel_default = str_apply(&FUEL_NAT_DEFAULT, &vec![ident_var(&id_fuel)]);
            let binder = ident_binder(&str_ident(FUEL_PARAM), &str_typ(FUEL_TYPE));
            mk_exists(&vec![binder], &vec![], &mk_eq(&fuel_default, &default_fuel))
        };
        let fuel_axiom = Rc::new(DeclX::Axiom(axiom_expr));
        commands.push(Rc::new(CommandX::Global(fuel_axiom)));
    }

    // non-recursive:
    //   (axiom (=> (fuel_bool fuel%f) (forall (...) (= (f ...) body))))
    // recursive:
    //   (axiom (forall (... fuel) (and
    //     (= (rec%f ... succ(fuel)) (rec%f ... zero) )
    //     (= (rec%f ... succ(fuel)) body[rec%f ... fuel] )
    //   )))
    //   (axiom (forall (...) (= (f ...) (rec%f ... (fuel_nat fuel%f)))))
    let body_expr = exp_to_expr(&ctx, &body_exp);
    if !is_recursive {
        let e_forall = func_def_quant(
            &suffix_global_id(&function.x.name),
            &function.x.typ_params,
            &function.x.params,
            body_expr,
        )?;
        let fuel_bool = str_apply(FUEL_BOOL, &vec![ident_var(&id_fuel)]);
        let imply = mk_implies(&fuel_bool, &e_forall);
        let def_axiom = Rc::new(DeclX::Axiom(imply));
        commands.push(Rc::new(CommandX::Global(def_axiom)));
    } else {
        let rec_f = suffix_global_id(&prefix_recursive(&function.x.name));
        let args = func_def_args(&function.x.typ_params, &function.x.params);
        let mut args_bump = args.clone();
        let mut args_succ = args.clone();
        let mut args_fuel = args.clone();
        // REVIEW: ZERO looks more efficient than FUEL_LOCAL, but less flexible.
        // Specifically, FUEL_LOCAL allows saying fuel >= value,
        // while ZERO only works with fuel == value.
        //   args_bump.push(str_var(ZERO));
        args_bump.push(str_var(FUEL_LOCAL));
        args_succ.push(str_apply(SUCC, &vec![str_var(FUEL_LOCAL)]));
        args_fuel.push(str_apply(FUEL_NAT, &vec![ident_var(&id_fuel)]));
        let rec_f_bump = ident_apply(&rec_f, &args_bump);
        let rec_f_succ = ident_apply(&rec_f, &args_succ);
        let rec_f_fuel = ident_apply(&rec_f, &args_fuel);
        let eq_bump = mk_eq(&rec_f_succ, &rec_f_bump);
        let eq_body = mk_eq(&rec_f_succ, &body_expr);
        let and = mk_and(&vec![eq_bump, eq_body]);
        let bind = func_bind(&function.x.typ_params, &function.x.params, &rec_f_succ, true);
        let rec_forall = Rc::new(ExprX::Bind(bind, and));
        let def_forall = func_def_quant(
            &suffix_global_id(&function.x.name),
            &function.x.typ_params,
            &function.x.params,
            rec_f_fuel,
        )?;
        let mut rec_typs = vec_map(&*function.x.params, |param| typ_to_air(&param.x.typ));
        rec_typs.push(str_typ(FUEL_TYPE));
        let rec_typ = typ_to_air(&function.x.ret.as_ref().unwrap().1);
        let rec_decl = Rc::new(DeclX::Fun(rec_f, Rc::new(rec_typs), rec_typ));
        let rec_axiom = Rc::new(DeclX::Axiom(rec_forall));
        let def_axiom = Rc::new(DeclX::Axiom(def_forall));
        commands.push(Rc::new(CommandX::Global(rec_decl)));
        commands.push(Rc::new(CommandX::Global(rec_axiom)));
        commands.push(Rc::new(CommandX::Global(def_axiom)));
    }
    Ok(())
}

pub fn req_ens_to_air(
    ctx: &Ctx,
    commands: &mut Vec<Command>,
    params: &Params,
    typing_invs: &Vec<Expr>,
    specs: &Vec<crate::ast::Expr>,
    typ_params: &Idents,
    typs: &air::ast::Typs,
    name: &Ident,
    msg: &Option<String>,
) -> Result<(), VirErr> {
    if specs.len() + typing_invs.len() > 0 {
        let mut all_typs = (**typs).clone();
        for _ in typ_params.iter() {
            all_typs.insert(0, str_typ(crate::def::TYPE));
        }
        let decl = Rc::new(DeclX::Fun(name.clone(), Rc::new(all_typs), bool_typ()));
        commands.push(Rc::new(CommandX::Global(decl)));
        let mut exprs: Vec<Expr> = Vec::new();
        for e in typing_invs {
            exprs.push(e.clone());
        }
        for e in specs.iter() {
            let exp = crate::ast_to_sst::expr_to_exp(ctx, e)?;
            let expr = exp_to_expr(ctx, &exp);
            let loc_expr = match msg {
                None => expr,
                Some(msg) => {
                    let description = Some(msg.clone());
                    let option_span = Rc::new(Some(Span { description, ..e.span.clone() }));
                    Rc::new(ExprX::LabeledAssertion(option_span, expr))
                }
            };
            exprs.push(loc_expr);
        }
        let body = Rc::new(ExprX::Multi(MultiOp::And, Rc::new(exprs)));
        let e_forall = func_def_quant(&name, &typ_params, &params, body)?;
        let req_ens_axiom = Rc::new(DeclX::Axiom(e_forall));
        commands.push(Rc::new(CommandX::Global(req_ens_axiom)));
    }
    Ok(())
}

pub fn func_decl_to_air(ctx: &Ctx, function: &Function) -> Result<Commands, VirErr> {
    let mut all_typs = vec_map(&function.x.params, |param| typ_to_air(&param.x.typ));
    let param_typs = Rc::new(all_typs.clone());
    for _ in function.x.typ_params.iter() {
        all_typs.insert(0, str_typ(crate::def::TYPE));
    }
    let mut commands: Vec<Command> = Vec::new();
    match (function.x.mode, function.x.ret.as_ref()) {
        (Mode::Spec, Some((_, ret, _))) => {
            let typ = typ_to_air(&ret);

            // Declare function
            let name = suffix_global_id(&function.x.name);
            let decl = Rc::new(DeclX::Fun(name.clone(), Rc::new(all_typs), typ));
            commands.push(Rc::new(CommandX::Global(decl)));

            // Body
            if let Some(body) = &function.x.body {
                func_body_to_air(ctx, &mut commands, function, body)?;
            }

            // Return typing invariant
            let mut f_args: Vec<Expr> = Vec::new();
            for typ_param in function.x.typ_params.iter() {
                f_args.push(ident_var(&suffix_typ_param_id(&typ_param.clone())));
            }
            for param in function.x.params.iter() {
                f_args.push(ident_var(&suffix_local_id(&param.x.name.clone())));
            }
            let f_app = ident_apply(&name, &Rc::new(f_args));
            if let Some(expr) = typ_invariant(&ret, &f_app, true) {
                // (axiom (forall (...) expr))
                let e_forall = Rc::new(ExprX::Bind(
                    func_bind(&function.x.typ_params, &function.x.params, &f_app, false),
                    expr,
                ));
                let inv_axiom = Rc::new(DeclX::Axiom(e_forall));
                commands.push(Rc::new(CommandX::Global(inv_axiom)));
            }
        }
        (Mode::Exec, _) | (Mode::Proof, _) => {
            req_ens_to_air(
                ctx,
                &mut commands,
                &function.x.params,
                &vec![],
                &function.x.require,
                &function.x.typ_params,
                &param_typs,
                &prefix_requires(&function.x.name),
                &Some("failed precondition".to_string()),
            )?;
            let mut ens_typs = (*param_typs).clone();
            let mut ens_params = (*function.x.params).clone();
            let mut ens_typing_invs: Vec<Expr> = Vec::new();
            if let Some((name, typ, mode)) = &function.x.ret {
                let param = ParamX { name: name.clone(), typ: typ.clone(), mode: *mode };
                ens_typs.push(typ_to_air(&typ));
                ens_params.push(Spanned::new(function.span.clone(), param));
                if let Some(expr) = typ_invariant(&typ, &ident_var(&suffix_local_id(&name)), true) {
                    ens_typing_invs.push(expr.clone());
                }
            }
            req_ens_to_air(
                ctx,
                &mut commands,
                &Rc::new(ens_params),
                &ens_typing_invs,
                &function.x.ensure,
                &function.x.typ_params,
                &Rc::new(ens_typs),
                &prefix_ensures(&function.x.name),
                &None,
            )?;
        }
        _ => {}
    }
    Ok(Rc::new(commands))
}

pub fn func_def_to_air(ctx: &Ctx, function: &Function) -> Result<Commands, VirErr> {
    match (function.x.mode, function.x.ret.as_ref(), function.x.body.as_ref()) {
        (Mode::Exec, _, Some(body)) | (Mode::Proof, _, Some(body)) => {
            let dest = function.x.ret.as_ref().map(|(x, _, _)| x.clone());
            let reqs =
                vec_map_result(&*function.x.require, |e| crate::ast_to_sst::expr_to_exp(ctx, e))?;
            let enss =
                vec_map_result(&*function.x.ensure, |e| crate::ast_to_sst::expr_to_exp(ctx, e))?;
            let stm = crate::ast_to_sst::expr_to_stm(&ctx, &body, &dest)?;
            let stm = crate::recursion::check_termination_stm(ctx, function, &stm)?;
            let commands = crate::sst_to_air::body_stm_to_air(
                ctx,
                &function.x.typ_params,
                &function.x.params,
                &function.x.ret,
                &function.x.hidden,
                &reqs,
                &enss,
                &stm,
            );
            Ok(commands)
        }
        _ => Ok(Rc::new(vec![])),
    }
}
