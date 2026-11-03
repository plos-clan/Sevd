use crate::compiler::com_error::ParserError;
use crate::compiler::ir::AstNode;
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::parser::Parser;
use crate::compiler::parser::define::var_parser;
use crate::compiler::parser::expr::{ExprTerminator, ExprType, get_of_end_or_block_end_expr};
use crate::compiler::parser::fors::for_parser;
use crate::compiler::parser::returns::return_parser;
use crate::compiler::parser::whiles::while_parser;

fn parse_statement(
    parser: &mut Parser,
    token: Token,
) -> Result<(Option<AstNode>, ExprTerminator), ParserError> {
    match token.get_type() {
        TokenType::Let => Ok((Some(var_parser(parser, token)?), ExprTerminator::End)),
        TokenType::For => Ok((Some(for_parser(parser)?), ExprTerminator::End)),
        TokenType::While => Ok((Some(while_parser(parser)?), ExprTerminator::End)),
        TokenType::Return => Ok((Some(return_parser(parser)?), ExprTerminator::End)),
        TokenType::End => Ok((None, ExprTerminator::End)),
        _ => {
            parser.cache = Some(token.clone());
            let (expr, term) = get_of_end_or_block_end_expr(parser, &token, ExprType::Init)?;
            Ok((Some(AstNode::Expr(Box::new(expr))), term))
        }
    }
}

pub fn block_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let mut nodes: Vec<AstNode> = vec![];
    let mut token = parser.get_token()?;
    let TokenType::Lp('{') = token.get_type() else {
        let err = token.clone();
        let (stmt, terminator) = parse_statement(parser, token)?;
        if matches!(terminator, ExprTerminator::BlockEnd) {
            return Err(ParserError::IllegalStmtEnd(err));
        };
        return Ok(AstNode::Block {
            body: nodes,
            tail: Some(Box::new(stmt.unwrap())),
        });
    };
    let mut tail = None;
    loop {
        token = parser.get_token()?;
        if let TokenType::Lr('}') = token.get_type() {
            break;
        }
        let (stmt, terminator) = parse_statement(parser, token)?;
        if matches!(terminator, ExprTerminator::BlockEnd) {
            tail = Some(Box::new(stmt.unwrap()));
            break;
        }
        if let Some(node) = stmt {
            nodes.push(node);
        }
    }
    Ok(AstNode::Block { body: nodes, tail })
}
