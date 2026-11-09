use crate::compiler::com_error::ParserError;
use crate::compiler::ir::AstNode;
use crate::compiler::lexer::TokenType;
use crate::compiler::lexer::{OperatorEnum, Token};

use super::Parser;

fn named_parse(parser: &mut Parser) -> Result<Vec<Token>, ParserError> {
    let mut token = parser.get_token()?;
    match token.get_type() {
        TokenType::Identifier => Ok(vec![token]),
        TokenType::Lp('{') => {
            let mut named = vec![];
            loop {
                token = parser.get_token()?;
                let TokenType::Identifier = token.get_type() else {
                    return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
                };
                named.push(token);
                token = parser.get_token()?;
                match token.get_type() {
                    TokenType::Lr('}') => break,
                    TokenType::Operator(OperatorEnum::Comma) => {}
                    _ => return Err(ParserError::ExpectedToken(token, TokenType::Lp('}'))),
                }
            }
            Ok(named)
        }
        _ => Err(ParserError::ExpectedToken(token, TokenType::Identifier)),
    }
}

pub fn import_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let imp_name = named_parse(parser)?;

    let mut token = parser.get_token()?;
    if !matches!(token.get_type(), TokenType::From) {
        return Err(ParserError::ExpectedToken(token, TokenType::From));
    }
    token = parser.get_token()?;

    let path_token = token.clone();
    let TokenType::String(_) = path_token.get_type() else {
        return Err(ParserError::ExpectedToken(
            token,
            TokenType::String("lib_path".to_string()),
        ));
    };
    Ok(AstNode::Import {
        name: imp_name,
        path: path_token,
    })
}
