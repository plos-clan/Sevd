use super::Parser;
use crate::compiler::com_error::ParserError;
use crate::compiler::lexer::{OperatorEnum, Token};
use crate::compiler::{ir::Pattern, lexer::TokenType};

fn parse_args_in_paren(parser: &mut Parser) -> Result<Vec<Pattern>, ParserError> {
    let mut args = Vec::new();
    let token = parser.get_token()?;
    if matches!(token.get_type(), TokenType::Lr(')')) {
        parser.cache = Some(token);
        return Ok(args);
    }
    parser.cache = Some(token);

    loop {
        let pat = pattern_parser(parser)?;
        args.push(pat);

        let sep = parser.get_token()?;
        match sep.get_type() {
            TokenType::Operator(OperatorEnum::Comma) => continue,
            TokenType::Lr(')') => {
                parser.cache = Some(sep);
                break;
            }
            _ => {
                return Err(ParserError::Expected(sep, ':'));
            }
        }
    }

    Ok(args)
}

fn parse_path_or_bind(parser: &mut Parser, first: Token) -> Result<Pattern, ParserError> {
    let mut path = vec![first];
    loop {
        let token = parser.get_token()?;
        if matches!(token.get_type(), TokenType::Operator(OperatorEnum::Path)) {
            let ident = parser.get_token()?;
            if !matches!(ident.get_type(), TokenType::Identifier) {
                return Err(ParserError::ExpectedToken(ident, TokenType::Identifier));
            }
            path.push(ident);
        } else {
            parser.cache = Some(token);
            break;
        }
    }
    let token = parser.get_token()?;

    if matches!(token.get_type(), TokenType::Lp('(')) {
        let args = parse_args_in_paren(parser)?;
        let close = parser.get_token()?;
        if !matches!(close.get_type(), TokenType::Lr(')')) {
            return Err(ParserError::ExpectedToken(close, TokenType::Lr(')')));
        }

        if path.len() == 1 {
            Ok(Pattern::Variant {
                name: path.into_iter().next().unwrap(),
                args,
            })
        } else {
            Ok(Pattern::Constructor { path, args })
        }
    } else {
        parser.cache = Some(token);
        Ok(Pattern::Bind(path.into_iter().next().unwrap()))
    }
}

fn parse_tuple(parser: &mut Parser) -> Result<Pattern, ParserError> {
    let first = pattern_parser(parser)?;

    let token = parser.get_token()?;
    if matches!(token.get_type(), TokenType::Operator(OperatorEnum::Comma)) {
        let mut elements = vec![first];

        loop {
            let pat = pattern_parser(parser)?;
            elements.push(pat);

            let sep = parser.get_token()?;
            match sep.get_type() {
                TokenType::Operator(OperatorEnum::Comma) => continue,
                TokenType::Lr(')') => break,
                _ => {
                    return Err(ParserError::ExpectedToken(sep, TokenType::Lr(')')));
                }
            }
        }
        Ok(Pattern::Tuple(elements))
    } else if matches!(token.get_type(), TokenType::Lr(')')) {
        Ok(first)
    } else {
        Err(ParserError::ExpectedToken(token, TokenType::Lr(')')))
    }
}

fn parse_pattern_no_or(parser: &mut Parser) -> Result<Pattern, ParserError> {
    let token = parser.get_token()?;

    match token.get_type() {
        TokenType::Identifier if token.get_span().text() == "_" => Ok(Pattern::Wildcard),
        TokenType::Number(_) | TokenType::String(_) => Ok(Pattern::Literal(token)),
        TokenType::Lp('(') => parse_tuple(parser),
        TokenType::Identifier => parse_path_or_bind(parser, token),
        _ => Err(ParserError::ExpectedToken(token, TokenType::Identifier)),
    }
}

pub fn pattern_parser(parser: &mut Parser) -> Result<Pattern, ParserError> {
    let pat = parse_pattern_no_or(parser)?;
    let mut patterns = vec![pat];

    loop {
        let token = parser.get_token()?;
        if !matches!(token.get_type(), TokenType::Operator(OperatorEnum::BitOr)) {
            parser.cache = Some(token);
            break;
        }

        let next_pat = parse_pattern_no_or(parser)?;
        patterns.push(next_pat);
    }

    if patterns.len() > 1 {
        Ok(Pattern::Or(patterns))
    } else {
        Ok(patterns.into_iter().next().unwrap())
    }
}
