use crate::compiler::com_error::ParserError;
use crate::compiler::ir::{AstNode, GuardNode};
use crate::compiler::lexer::TokenType;
use crate::compiler::parser::block::block_parser;
use crate::compiler::parser::guard::parse_guard_chain;
use crate::compiler::parser::Parser;

pub fn while_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let token = parser.get_token()?;
    if matches!(token.get_type(), TokenType::Lp('{')) {
        parser.cache = Some(token);
        return Ok(AstNode::Loop {
            body: Box::new(block_parser(parser)?),
        });
    }

    parser.cache = Some(token);
    let conditions = parse_guard_chain(parser, ParserError::MissingLoopBody)?;
    let body = Box::new(block_parser(parser)?);

    if conditions.len() == 1 {
        return match conditions.into_iter().next().unwrap() {
            GuardNode::Expr(cond) => Ok(AstNode::While {
                cond: Box::new(cond),
                body,
            }),
            node => Ok(AstNode::WhilePattern {
                patterns: vec![node],
                body,
            }),
        };
    }

    Ok(AstNode::WhilePattern {
        patterns: conditions,
        body,
    })
}
