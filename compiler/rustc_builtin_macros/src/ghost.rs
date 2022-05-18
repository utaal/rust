use rustc_ast as ast;
use rustc_ast::ptr::P;
use rustc_ast::token::{self, Delimiter};
use rustc_ast::tokenstream::TokenStream;
use rustc_expand::base::{self, *};
use rustc_parse::parser::Parser;
use rustc_parse::parser::AttemptLocalParseRecovery;
// use rustc_parse::{self, new_parser_from_file};
use rustc_session::lint::builtin::INCOMPLETE_INCLUDE;
use rustc_session::parse::ParseSess;
use rustc_span::symbol::Ident;
use rustc_span::symbol::{kw, sym, Symbol};
use rustc_span::{self, Pos, Span};
use rustc_ast::ast::{BlockCheckMode, GhostMode};
use rustc_errors::{Applicability, PResult};

pub struct GhostArgs(Ident, GhostMode);

fn parse_args<'a>(
    p: &mut Parser<'a>,
    sess: &'a ParseSess,
    sp: Span,
) -> PResult<'a, GhostArgs> {
    p.expect(&token::OpenDelim(Delimiter::Parenthesis))?;

    let id = p.parse_ident()?;

    p.expect(&token::TokenKind::Comma)?;

    let g = if p.eat_keyword(sym::regular) {
        GhostMode::Regular
    } else if p.eat_keyword(sym::no_init) {
        GhostMode::NoInit
    } else {
        let err = p.sess.span_diagnostic.struct_span_err(
            p.token.span,
            "ghost mode should be either `regular` or `no_init`",
        );
        return Err(err);
    };

    p.expect(&token::CloseDelim(Delimiter::Parenthesis))?;

    Ok(GhostArgs(id, g))
}

pub fn expand_ghost(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
) -> Box<dyn base::MacResult + 'static> {
    let mut p = cx.new_parser_from_tts(tts);
    let sess = &cx.sess.parse_sess;

    match parse_args(&mut p, sess, sp) {
        Ok(GhostArgs(id, g)) => {

            let mut stmts = vec![];

            // TODO move to parser?
            while !p.eat(&token::CloseDelim(Delimiter::Brace)) {
                if p.token == token::Eof {
                    break;
                }
                let stmt = match p.parse_full_stmt(AttemptLocalParseRecovery::No) {
                    Ok(stmt) => stmt,
                    Err(mut err) => {
                        err.emit();
                        return DummyResult::any(sp);
                    }
                };
                if let Some(stmt) = stmt {
                    stmts.push(stmt);
                } else {
                    // Found only `;` or `}`.
                    continue;
                };
            }

            let block = P(ast::Block {
                stmts,
                id: ast::DUMMY_NODE_ID,
                rules: BlockCheckMode::Ghost(id, g),
                span: sp,
                tokens: None,
                could_be_bare_literal: false,
            });
            base::MacEager::expr(
                cx.expr(
                    sp,
                    ast::ExprKind::Block(block, None),
                    ))
        }
        Err(mut err) => {
            err.emit();
            DummyResult::any(sp)
        }
    }
    
}
