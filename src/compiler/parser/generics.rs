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

/// 用于引用语法, 如 Ident<Struct<i32,i64>>/i32/(usize, i32)
pub fn parser_type_ref(parser: &mut Parser) -> Result<GenericArg, ParserError> {
    let mut token = parser.get_token()?;

    if let TokenType::Lp('(') = token.get_type() {
        let mut elements = vec![];
        loop {
            let ty = parser_type_ref(parser)?;
            elements.push(ty);
            token = parser.get_token()?;
            match token.get_type() {
                TokenType::Operator(OperatorEnum::Comma) => continue,
                TokenType::Lr(')') => break,
                _ => return Err(ParserError::Expected(token, ')')),
            }
        }
        return if elements.len() == 1 {
            Ok(elements.into_iter().next().unwrap())
        } else {
            Ok(GenericArg::Tuple(elements))
        };
    }

    let TokenType::Identifier = token.get_type() else {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    };
    let name = token;
    token = parser.get_token()?;
    if let TokenType::Operator(OperatorEnum::Less) = token.get_type() {
        let mut generics = vec![];
        loop {
            let ga = parser_type_ref(parser)?;
            generics.push(ga);
            token = parser.get_token()?;
            match token.get_type() {
                TokenType::Operator(OperatorEnum::Comma) => continue,
                TokenType::Operator(OperatorEnum::Big) => break,
                _ => return Err(ParserError::Expected(token, '>')),
            }
        }
        Ok(GenericArg::Named { name, generics })
    } else {
        parser.cache = Some(token);
        Ok(GenericArg::Ident(name))
    }
}

fn parser_constraint_unit(
    parser: &mut Parser,
) -> Result<Option<(Token, Vec<GenericArg>)>, ParserError> {
    let mut token = parser.get_token()?;
    let TokenType::Identifier = token.get_type() else {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    };
    let name = token;
    token = parser.get_token()?;
    let TokenType::Operator(OperatorEnum::Colon) = token.get_type() else {
        return Err(ParserError::Expected(token, ':'));
    };
    let mut generics = vec![];
    loop {
        let generic = parser_type_ref(parser)?;
        generics.push(generic);
        token = parser.get_token()?;
        let TokenType::Operator(OperatorEnum::Add) = token.get_type() else {
            parser.cache = Some(token);
            break;
        };
    }
    Ok(Some((name, generics)))
}

pub fn parser_constraint(
    parser: &mut Parser,
) -> Result<Vec<(Token, Vec<GenericArg>)>, ParserError> {
    let mut constraints = vec![];
    while let Some(constraint) = parser_constraint_unit(parser)? {
        constraints.push(constraint);
        let token = parser.get_token()?;
        match token.get_type() {
            TokenType::Lp('{') => {
                parser.cache = Some(token);
                break;
            }
            TokenType::Operator(OperatorEnum::Comma) => continue,
            _ => return Err(ParserError::Expected(token, ',')),
        }
    }
    Ok(constraints)
}
