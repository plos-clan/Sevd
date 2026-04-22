use std::vec;

use crate::compiler::com_error::ParserError;
use crate::compiler::ir::GenericArg;
use crate::compiler::lexer::{OperatorEnum, Token, TokenType};
use crate::compiler::parser::Parser;

/// 用于解析泛型定义语法，如 <T,K>
pub fn parser_generics(parser: &mut Parser) -> Result<Vec<Token>, ParserError> {
    let mut tokens = vec![];
    loop {
        let mut token = parser.get_token()?;
        let TokenType::Identifier = token.get_type() else {
            return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
        };
        tokens.push(token);
        token = parser.get_token()?;

        match token.get_type() {
            TokenType::Operator(OperatorEnum::Comma) => continue,
            TokenType::Operator(OperatorEnum::Big) => break,
            _ => return Err(ParserError::Expected(token, '>')),
        }
    }
    Ok(tokens)
}

/// 用于解析泛型使用语法, 如 Ident<Struct<i32,i64>>
pub fn parser_generics_use(parser: &mut Parser) -> Result<GenericArg, ParserError> {
    let mut token = parser.get_token()?;
    let TokenType::Identifier = token.get_type() else {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    };
    let name = token;
    token = parser.get_token()?;
    if let TokenType::Operator(OperatorEnum::Less) = token.get_type() {
        let mut generics = vec![];
        loop {
            let ga = parser_generics_use(parser)?;
            generics.push(ga);
            token = parser.get_token()?;
            match token.get_type() {
                TokenType::Operator(OperatorEnum::Comma) => continue,
                TokenType::Operator(OperatorEnum::Big) => break,
                _ => return Err(ParserError::Expected(token, '>')),
            }
        }
        Ok(GenericArg::Named {
            name,
            generics,
        })
    } else {
        parser.cache = Some(token);
        Ok(GenericArg::Ident(name))
    }
}
