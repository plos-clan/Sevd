use crate::compiler::com_error::ParserError;
use crate::compiler::ir::{AstNode, Pattern};
use crate::compiler::lexer::{OperatorEnum, Token, TokenType};
use crate::compiler::parser::function::parser_generics;
use crate::compiler::parser::pattern::pattern_parser;
use crate::compiler::parser::Parser;

use super::block::block_parser;
use super::expr::get_of_else_end_expr;

fn end_var_parser(
    parser: &mut Parser,
    has_type: bool,
    head: Pattern,
    last: Token,
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
                return Ok(AstNode::DefineElse {
                    head,
                    type_name: Some(type_name),
                    vars: None,
                    el_blk: None,
                });
            }
            _ => return Err(ParserError::Expected(set_opt, '=')),
        }
    } else {
        None
    };
    let expr = get_of_else_end_expr(parser, &last)?;

    let else_key = parser.get_token()?;
    let el_blk: Option<Box<AstNode>> = if let TokenType::Else = else_key.get_type() {
        Some(Box::new(block_parser(parser)?))
    } else {
        parser.cache = Some(else_key);
        None
    };

    Ok(AstNode::DefineElse {
        head,
        type_name,
        vars: Some(Box::new(expr)),
        el_blk,
    })
}

pub fn var_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let pattern = pattern_parser(parser)?;

    let token = parser.get_token()?;

    match token.get_type() {
        TokenType::Operator(OperatorEnum::Colon) => end_var_parser(parser, true, pattern, token),
        TokenType::Operator(OperatorEnum::Set) => end_var_parser(parser, false, pattern, token),
        TokenType::End => Ok(AstNode::DefineElse {
            head: pattern,
            type_name: None,
            vars: None,
            el_blk: None,
        }),
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
                parser.cache = Some(token);
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

fn field_parser(parser: &mut Parser) -> Result<Vec<(Token, Token)>, ParserError> {
    let mut token = parser.get_token()?;
    let mut fields = Vec::new();
    if !matches!(token.get_type(), TokenType::Lp('{')) {
        return Err(ParserError::Expected(token, '{'));
    }
    loop {
        token = parser.get_token()?;
        if matches!(token.get_type(), TokenType::Lr('}')) {
            break;
        }
        if !matches!(token.get_type(), TokenType::Identifier) {
            return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
        }
        let name = token;
        token = parser.get_token()?;
        if !matches!(token.get_type(), TokenType::Operator(OperatorEnum::Colon)) {
            return Err(ParserError::Expected(token, ':'));
        }
        token = parser.get_token()?;
        if !matches!(token.get_type(), TokenType::Identifier) {
            return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
        }
        fields.push((name, token));
        token = parser.get_token()?;
        match token.get_type() {
            TokenType::Operator(OperatorEnum::Comma) => continue,
            TokenType::Lr('}') => break,
            _ => return Err(ParserError::Expected(token, '}')),
        }
    }

    Ok(fields)
}

pub fn struct_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let mut token = parser.get_token()?;
    if !matches!(token.get_type(), TokenType::Identifier) {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    }
    let name = token;
    token = parser.get_token()?;
    let generics = if matches!(token.get_type(), TokenType::Operator(OperatorEnum::Less)) {
        Some(parser_generics(parser)?)
    } else if matches!(token.get_type(), TokenType::Lp('{')) {
        parser.cache = Some(token);
        None
    } else {
        return Err(ParserError::Expected(token, '{'));
    };
    let fields = field_parser(parser)?;
    Ok(AstNode::StructDefine {
        name,
        generics,
        fields,
    })
}
