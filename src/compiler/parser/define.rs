use crate::compiler::com_error::ParserError;
use crate::compiler::ir::{AstNode, Pattern};
use crate::compiler::lexer::{OperatorEnum, Token, TokenType};
use crate::compiler::parser::Parser;
use crate::compiler::parser::expr::get_of_end_expr;

use super::block::block_parser;
use super::expr::get_of_else_end_expr;

fn irrefutable_pattern_parser(parser: &mut Parser, name: Token) -> Result<AstNode, ParserError> {
    let mut args = vec![];
    loop {
        let mut token = parser.get_token()?;
        if !matches!(
            token.get_type(),
            TokenType::Identifier
                | TokenType::Number(_)
                | TokenType::String(_)
                | TokenType::Null
                | TokenType::True
                | TokenType::False,
        ) {
            return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
        }
        args.push(Pattern::Bind(token));
        token = parser.get_token()?;
        match token.get_type() {
            TokenType::Operator(OperatorEnum::Comma) => {}
            TokenType::Lr(')') => break,
            _ => return Err(ParserError::Expected(token, ')')),
        }
    }

    let mut token = parser.get_token()?;
    let TokenType::Operator(OperatorEnum::Set) = token.get_type() else {
        return Err(ParserError::Expected(token, '='));
    };

    let expr = get_of_else_end_expr(parser, &name)?;

    token = parser.get_token()?;
    let el_blk: Option<Vec<AstNode>> = if let TokenType::Else = token.get_type() {
        Some(block_parser(parser)?)
    } else {
        parser.cache = Some(token);
        None
    };

    Ok(AstNode::DefineElse {
        head: Pattern::Variant { name, args },
        type_name: None,
        vars: Some(expr),
        el_blk,
    })
}

fn classic_var_parser(
    parser: &mut Parser,
    name: Token,
    has_type: bool,
) -> Result<AstNode, ParserError> {
    let type_name = if has_type {
        let type_name = parser.get_token()?;
        if !matches!(type_name.get_type(), TokenType::Identifier) {
            return Err(ParserError::ExpectedToken(type_name, TokenType::Identifier));
        }
        let set_opt = parser.get_token()?;
        match set_opt.get_type() {
            TokenType::Operator(OperatorEnum::Set) => Some(type_name),
            TokenType::End => {
                return Ok(AstNode::Define {
                    name,
                    type_name: Some(type_name),
                    vars: None,
                });
            }
            _ => return Err(ParserError::Expected(set_opt, '=')),
        }
    } else {
        None
    };
    let expr = get_of_end_expr(parser, &name)?;
    Ok(AstNode::Define {
        name,
        type_name,
        vars: Some(expr),
    })
}

pub fn var_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let name = parser.get_token()?;
    if !matches!(name.get_type(), TokenType::Identifier) {
        return Err(ParserError::ExpectedToken(name, TokenType::Identifier));
    }
    let token = parser.get_token()?;

    match token.get_type() {
        TokenType::Operator(OperatorEnum::Colon) => classic_var_parser(parser, name, true),
        TokenType::Operator(OperatorEnum::Set) => classic_var_parser(parser, name, false),
        TokenType::End => Ok(AstNode::Define {
            name,
            type_name: None,
            vars: None,
        }),
        TokenType::Lp('(') => irrefutable_pattern_parser(parser, name),
        _ => Err(ParserError::Expected(token, '=')),
    }
}

fn parser_enum_fields(parser: &mut Parser) -> Result<Vec<Token>, ParserError> {
    let mut enum_fields = vec![];
    loop {
        let mut token = parser.get_token()?;
        match token.get_type() {
            TokenType::Identifier => {
                enum_fields.push(token);
                token = parser.get_token()?;
                match token.get_type() {
                    TokenType::Operator(OperatorEnum::Comma) => continue,
                    TokenType::Lr(')') => break,
                    _ => return Err(ParserError::Expected(token, ')')),
                }
            }
            _ => return Err(ParserError::ExpectedToken(token, TokenType::Identifier)),
        }
    }
    Ok(enum_fields)
}

fn parser_enum_element(parser: &mut Parser) -> Result<Option<(Token, Vec<Token>)>, ParserError> {
    let mut token = parser.get_token()?;
    match token.get_type() {
        TokenType::Lr('}') => Ok(None),
        TokenType::Identifier => {
            let name = token;
            token = parser.get_token()?;
            let enum_fields = if let TokenType::Lp('(') = token.get_type() {
                parser_enum_fields(parser)?
            } else {
                vec![]
            };
            Ok(Some((name, enum_fields)))
        }
        _ => Err(ParserError::ExpectedToken(token, TokenType::Identifier)),
    }
}

pub fn enum_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let mut token = parser.get_token()?;
    if !matches!(token.get_type(), TokenType::Identifier) {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    }
    let name = token;
    token = parser.get_token()?;
    let TokenType::Lp('{') = token.get_type() else {
        return Err(ParserError::Expected(token, '{'));
    };
    let mut variants = Vec::new();
    while let Some(field) = parser_enum_element(parser)? {
        variants.push(field);
    }
    if variants.is_empty() {
        return Err(ParserError::MissingEnumElement(token));
    }
    Ok(AstNode::EnumDefine { name, variants })
}
