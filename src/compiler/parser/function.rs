use crate::compiler::com_error::ParserError;
use crate::compiler::ir::AstNode;
use crate::compiler::lexer::{OperatorEnum, Token, TokenType};
use crate::compiler::parser::Parser;

use super::block::block_parser;

fn parser_argument(parser: &mut Parser) -> Result<Vec<AstNode>, ParserError> {
    let mut nodes = vec![];
    loop {
        let mut token = parser.get_token()?;
        match token.get_type() {
            TokenType::Operator(OperatorEnum::Comma) => continue,
            TokenType::Lr(')') => break,
            TokenType::Identifier => {
                let name = token;
                token = parser.get_token()?;
                let TokenType::Operator(OperatorEnum::Colon) = token.get_type() else {
                    return Err(ParserError::Expected(token, ':'));
                };
                token = parser.get_token()?;
                if !matches!(token.get_type(), TokenType::Identifier) {
                    return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
                }
                nodes.push(AstNode::Define {
                    name,
                    type_name: Some(token),
                    vars: None,
                });
                continue;
            }
            _ => return Err(ParserError::ExpectedToken(token, TokenType::Identifier)),
        }
    }
    Ok(nodes)
}

fn parser_generics(parser: &mut Parser) -> Result<Vec<Token>, ParserError> {
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

pub fn function_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let mut token = parser.get_token()?;

    let (generics, name) = if matches!(token.get_type(), TokenType::Operator(OperatorEnum::Less)) {
        let generics = parser_generics(parser)?;
        token = parser.get_token()?;
        if !matches!(token.get_type(), TokenType::Identifier) {
            return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
        }
        (Some(generics), token)
    } else if matches!(token.get_type(), TokenType::Identifier) {
        (None, token)
    } else {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    };

    token = parser.get_token()?;

    let args: Vec<AstNode> = match token.get_type() {
        TokenType::Lp('(') => {
            let args = parser_argument(parser)?;
            token = parser.get_token()?;
            if let TokenType::Operator(OperatorEnum::Colon) = token.get_type() {
                args
            } else {
                return Err(ParserError::Expected(token, ':'));
            }
        }
        TokenType::Operator(OperatorEnum::Colon) => {
            vec![]
        }
        _ => return Err(ParserError::Expected(token, '(')),
    };

    token = parser.get_token()?;

    if !matches!(token.get_type(), TokenType::Identifier) {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    }
    let ret_type = token;
    Ok(AstNode::Function {
        name,
        generics,
        args,
        ret_type,
        block: block_parser(parser)?,
    })
}
