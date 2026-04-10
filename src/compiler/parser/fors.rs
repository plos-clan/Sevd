use crate::compiler::com_error::ParserError;
use crate::compiler::lexer::{Token, TokenType};
use crate::compiler::{ir::AstNode, parser::Parser};

fn irrefutable_pattern_parser(parser: &mut Parser, name: Token) -> Result<AstNode, ParserError> {
    todo!()
}

fn classic_for_parser(parser: &mut Parser, name: Token) -> Result<AstNode, ParserError> {
    todo!()
}

pub fn for_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let mut token = parser.get_token()?;

    if !matches!(token.get_type(), TokenType::Identifier) {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    }
    let name = token;
    token = parser.get_token()?;

    match token.get_type() {
        TokenType::Lp('(') => irrefutable_pattern_parser(parser, name),
        TokenType::From => classic_for_parser(parser, name),
        _ => Err(ParserError::ExpectedToken(token, TokenType::From)),
    }
}
