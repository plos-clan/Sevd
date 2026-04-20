use crate::compiler::com_error::ParserError;
use crate::compiler::ir::AstNode;
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::Parser;
use crate::compiler::parser::define::var_parser;
use crate::compiler::parser::expr::get_of_end_expr;
use crate::compiler::parser::fors::for_parser;
use crate::compiler::parser::whils::while_parser;

fn parse_statement(parser: &mut Parser, token: Token) -> Result<Option<AstNode>, ParserError> {
    match token.get_type() {
        TokenType::Let => Ok(Some(var_parser(parser)?)),
        TokenType::For => Ok(Some(for_parser(parser)?)),
        TokenType::While => Ok(Some(while_parser(parser)?)),
        TokenType::End => Ok(None),
        _ => {
            parser.cache = Some(token.clone());
            Ok(Some(AstNode::Expr(get_of_end_expr(parser, &token)?)))
        }
    }
}

pub fn block_parser(parser: &mut Parser) -> Result<Vec<AstNode>, ParserError> {
    let mut nodes: Vec<AstNode> = vec![];
    let mut token = parser.get_token()?;
    let TokenType::Lp('{') = token.get_type() else {
        if let Some(stmt) = parse_statement(parser, token.clone())? {
            nodes.push(stmt);
        }
        return Ok(nodes);
    };
    loop {
        token = parser.get_token()?;
        if let TokenType::Lr('}') = token.get_type() {
            break;
        }
        if let Some(node) = parse_statement(parser, token)? {
            nodes.push(node);
        }
    }
    Ok(nodes)
}
