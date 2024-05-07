#![allow(unused_imports)]

use rustc_ast as ast;
use rustc_ast::ptr::P;
use rustc_ast::token::{self, Delimiter};
use rustc_ast::tokenstream::TokenStream;
use rustc_expand::base::{self, *};
// use rustc_parse::parser::Parser;
use rustc_parse::parser::AttemptLocalParseRecovery;
// use rustc_parse::{self, new_parser_from_file};
// use rustc_session::lint::builtin::INCOMPLETE_INCLUDE;
// use rustc_session::parse::ParseSess;
use rustc_span::symbol::Ident;
// use rustc_span::symbol::{kw, sym, Symbol};
use rustc_span::{self, Span};
use rustc_ast::ast::{BlockCheckMode};
use thin_vec::{ThinVec, thin_vec};
// use rustc_errors::{Applicability, PResult};

// fn parse_args<'a>(
//     p: &mut Parser<'a>,
//     sess: &'a ParseSess,
//     sp: Span,
// ) -> PResult<'a, GhostArgs> {
//     p.expect(&token::OpenDelim(Delimiter::Parenthesis))?;
// 
//     let id = p.parse_ident()?;
// 
//     p.expect(&token::TokenKind::Comma)?;
// 
//     let g = if p.eat_keyword(sym::regular) {
//         GhostMode::Regular
//     } else if p.eat_keyword(sym::no_init) {
//         GhostMode::NoInit
//     } else {
//         let err = p.sess.span_diagnostic.struct_span_err(
//             p.token.span,
//             "ghost mode should be either `regular` or `no_init`",
//         );
//         return Err(err);
//     };
// 
//     p.expect(&token::CloseDelim(Delimiter::Parenthesis))?;
// 
//     Ok(GhostArgs(id, g))
// }

pub fn expand_ghost(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> MacroExpanderResult<'static> {

    let mut p = cx.new_parser_from_tts(tts);
    // let sess = &cx.sess.parse_sess;

    let mut stmts = ThinVec::new();

    // TODO move to parser?
    while !p.eat(&token::CloseDelim(Delimiter::Brace)) {
        if p.token == token::Eof {
            break;
        }
        let stmt = match p.parse_full_stmt(AttemptLocalParseRecovery::No) {
            Ok(stmt) => stmt,
            Err(err) => {
                let eg = err.emit();
                return ExpandResult::Ready(DummyResult::any(sp, eg));
            }
        };
        if let Some(stmt) = stmt {
            stmts.push(stmt);
        } else {
            // Found only `;` or `}`.
            continue;
        };
    }


    // let block_expr = cx.expr(
    //     sp,
    //     ast::ExprKind::Block(cx.block(sp, stmts), None),
    // );
    let block = cx.block(sp, stmts);

    // let ghost_path = ast::Path {
    //     span: sp,
    //     segments: thin_vec![
    //         ast::PathSegment::from_ident(Ident::from_str("core")),
    //         ast::PathSegment::from_ident(Ident::from_str("marker")),
    //         ast::PathSegment::from_ident(Ident::from_str("Ghost")),
    //         ast::PathSegment::from_ident(Ident::from_str("new")),
    //     ],
    //     tokens: None,
    // };
    // let path = cx.expr_path(ghost_path);

    // let call = cx.expr_call(sp, path, thin_vec![block_expr]);
    // let call = cx.stmt_expr(call);

    // let outer_block = P(ast::Block {
    //     stmts: thin_vec![block],
    //     id: ast::DUMMY_NODE_ID,
    //     rules: BlockCheckMode::Ghost,
    //     span: sp,
    //     tokens: None,
    //     could_be_bare_literal: false,
    // });

    ExpandResult::Ready(base::MacEager::expr(cx.expr_block(block)))
}