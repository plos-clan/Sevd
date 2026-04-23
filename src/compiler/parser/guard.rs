use crate::compiler::com_error::ParserError;
use crate::compiler::ir::GuardNode;
use crate::compiler::lexer::{OperatorEnum, Token, TokenType};
use crate::compiler::parser::Parser;
use crate::compiler::parser::expr::ExprParser;
use crate::compiler::parser::pattern::pattern_parser;

use super::expr::ExprType;

fn parse_condition_expr(
    parser: &mut Parser,
    fallback_token: Token,
) -> Result<GuardNode, ParserError> {
    let mut exprs = ExprParser::new(parser, fallback_token, ExprType::Cond);
    Ok(GuardNode::Expr(exprs.parse()?))
}

fn parse_let_condition(parser: &mut Parser) -> Result<GuardNode, ParserError> {
    let head = pattern_parser(parser)?;
    let set_token = parser.get_token()?;
    let TokenType::Operator(OperatorEnum::Set) = set_token.get_type() else {
        return Err(ParserError::Expected(set_token, '='));
    };

    let mut exprs = ExprParser::new(parser, set_token, ExprType::Cond);
    let vars = exprs.parse()?;
    Ok(GuardNode::Let { head, vars })
}

fn parse_condition_tokens(tokens: Vec<Token>) -> Result<GuardNode, ParserError> {
    let mut parser = Parser::new_collect(tokens);
    let first = parser.get_token()?;
    match first.get_type() {
        TokenType::Eof => Err(ParserError::MissingCondition(first)),
        TokenType::Let => parse_let_condition(&mut parser),
        _ => {
            parser.cache = Some(first.clone());
            parse_condition_expr(&mut parser, first)
        }
    }
}

fn is_top_level(brace_depth: usize, bracket_depth: usize, paren_depth: usize) -> bool {
    brace_depth == 0 && bracket_depth == 0 && paren_depth == 0
}

fn collect_conditions(
    parser: &mut Parser,
    missing_body: fn(Token) -> ParserError,
) -> Result<Vec<Vec<Token>>, ParserError> {
    let first = parser.get_token()?;
    let mut conditions = vec![];
    let mut current = vec![first.clone()];
    let mut brace_depth = usize::from(matches!(first.get_type(), TokenType::Lp('{')));
    let mut bracket_depth = usize::from(matches!(first.get_type(), TokenType::Lp('[')));
    let mut paren_depth = usize::from(matches!(first.get_type(), TokenType::Lp('(')));
    let mut pending_expr_block = matches!(first.get_type(), TokenType::From);

    loop {
        let token = parser.get_token()?;
        match token.get_type() {
            TokenType::Eof | TokenType::End
                if is_top_level(brace_depth, bracket_depth, paren_depth) =>
            {
                return Err(missing_body(first));
            }
            TokenType::Lp('{')
                if is_top_level(brace_depth, bracket_depth, paren_depth) && !pending_expr_block =>
            {
                if current.is_empty() {
                    return Err(ParserError::MissingCondition(token));
                }
                conditions.push(current);
                parser.cache = Some(token);
                return Ok(conditions);
            }
            TokenType::Lp('{') => {
                brace_depth += 1;
                current.push(token);
                pending_expr_block = false;
            }
            TokenType::Lr('}') => {
                if brace_depth == 0 {
                    return Err(ParserError::IllegalExpression(token));
                }
                brace_depth -= 1;
                current.push(token);
                pending_expr_block = false;
            }
            TokenType::Lp('[') => {
                bracket_depth += 1;
                current.push(token);
                pending_expr_block = false;
            }
            TokenType::Lr(']') => {
                if bracket_depth == 0 {
                    return Err(ParserError::IllegalExpression(token));
                }
                bracket_depth -= 1;
                current.push(token);
                pending_expr_block = false;
            }
            TokenType::Lp('(') => {
                paren_depth += 1;
                current.push(token);
                pending_expr_block = false;
            }
            TokenType::Lr(')') => {
                if paren_depth == 0 {
                    return Err(ParserError::IllegalExpression(token));
                }
                paren_depth -= 1;
                current.push(token);
                pending_expr_block = false;
            }
            TokenType::Operator(OperatorEnum::And)
                if is_top_level(brace_depth, bracket_depth, paren_depth) =>
            {
                if current.is_empty() {
                    return Err(ParserError::MissingCondition(token));
                }
                conditions.push(std::mem::take(&mut current));
                pending_expr_block = false;
            }
            TokenType::From => {
                current.push(token);
                pending_expr_block = true;
            }
            _ => {
                current.push(token);
                pending_expr_block = false;
            }
        }
    }
}

pub fn parse_guard_chain(
    parser: &mut Parser,
    missing_body: fn(Token) -> ParserError,
) -> Result<Vec<GuardNode>, ParserError> {
    collect_conditions(parser, missing_body)?
        .into_iter()
        .map(parse_condition_tokens)
        .collect()
}
