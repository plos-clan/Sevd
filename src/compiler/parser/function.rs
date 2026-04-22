use crate::compiler::com_error::ParserError;
use crate::compiler::ir::AstNode;
use crate::compiler::lexer::{OperatorEnum, TokenType};
use crate::compiler::parser::generics::parser_generics;
use crate::compiler::parser::Parser;

use super::block::block_parser;
use super::generics::parser_generics_use;

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

                let generices = parser_generics_use(parser)?;

                nodes.push(AstNode::Define {
                    name,
                    type_name: Some(generices),
                });
                continue;
            }
            _ => return Err(ParserError::ExpectedToken(token, TokenType::Identifier)),
        }
    }
    Ok(nodes)
}

pub fn function_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let mut token = parser.get_token()?;
    if !matches!(token.get_type(), TokenType::Identifier) {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    }
    let name = token;
    token = parser.get_token()?;
    let generics = if matches!(token.get_type(), TokenType::Operator(OperatorEnum::Less)) {
        let generics = parser_generics(parser)?;
        Some(generics)
    } else if matches!(
        token.get_type(),
        TokenType::Lp('(' | '{') | TokenType::Operator(OperatorEnum::Colon)
    ) {
        parser.cache = Some(token);
        None
    } else {
        return Err(ParserError::MissingFunctionBody(token));
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

    let ret_type = parser_generics_use(parser)?;

    Ok(AstNode::Function {
        name,
        generics,
        args,
        ret_type,
        block: Box::new(block_parser(parser)?),
    })
}
