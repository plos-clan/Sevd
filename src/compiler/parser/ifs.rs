use crate::compiler::com_error::ParserError;
use crate::compiler::ir::{AstNode, ExprNode};
use crate::compiler::lexer::TokenType;
use crate::compiler::parser::block::block_parser;
use crate::compiler::parser::guard::parse_guard_chain;
use crate::compiler::parser::Parser;

pub fn if_parser(parser: &mut Parser) -> Result<ExprNode, ParserError> {
    let branches = parse_guard_chain(parser, ParserError::MissingStatement)?;
    let body = block_parser(parser)?;

    let token = parser.get_token()?;
    let else_body = match token.get_type() {
        TokenType::Else => Some(block_parser(parser)?),
        TokenType::Elif => {
            let elif_node = if_parser(parser)?;
            Some(AstNode::Expr(Box::new(elif_node)))
        }
        _ => {
            parser.cache = Some(token);
            None
        }
    };

    Ok(ExprNode::IfPattern {
        branches,
        body,
        else_body,
    })
}
