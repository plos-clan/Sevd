use crate::compiler::com_error::ParserError;
use crate::compiler::lexer::{OperatorEnum, TokenType};
use crate::compiler::parser::pattern::pattern_parser;
use crate::compiler::{ir::AstNode, parser::Parser};

use super::block::block_parser;
use super::expr::{ExprParser, ExprType};

pub fn for_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let pattern = pattern_parser(parser)?;
    let mut token = parser.get_token()?;

    let exit = match token.get_type() {
        TokenType::From => {
            parser.cache = Some(token);
            false
        }
        TokenType::Operator(OperatorEnum::Colon) => {
            token = parser.get_token()?;
            match token.get_type() {
                TokenType::Break => true,
                TokenType::Continue => false,
                _ => return Err(ParserError::ExpectedToken(token, TokenType::Break)),
            }
        }
        _ => return Err(ParserError::ExpectedToken(token, TokenType::From)),
    };

    token = parser.get_token()?;
    let TokenType::From = token.get_type() else {
        return Err(ParserError::ExpectedToken(token, TokenType::From));
    };

    let mut expr_parser = ExprParser::new(parser, token, ExprType::Cond);
    let iter = Box::new(expr_parser.parse()?);
    let blk = Box::new(block_parser(parser)?);

    Ok(AstNode::ForPattern {
        pattern,
        exit,
        iter,
        blk,
    })
}
