use super::Parser;
use crate::compiler::com_error::ParserError;
use crate::compiler::ir::{AnnotationElement, AstNode, MetaValue};
use crate::compiler::lexer::{OperatorEnum, Token, TokenType};

fn parse_meta_value(token: Token) -> Result<MetaValue, ParserError> {
    match token.get_type() {
        TokenType::Identifier => Ok(MetaValue::Identifier(token)),
        TokenType::String(_)
        | TokenType::Char(_)
        | TokenType::Number(_)
        | TokenType::True
        | TokenType::False
        | TokenType::Null => Ok(MetaValue::Literal(token)),
        _ => Err(ParserError::IllegalKey(token)),
    }
}

fn element_parser(parser: &mut Parser) -> Result<Vec<AnnotationElement>, ParserError> {
    let token = parser.get_token()?;
    let mut elements = vec![];
    let TokenType::Lp('(') = token.get_type() else {
        parser.cache = Some(token);
        return Ok(elements);
    };

    loop {
        let token = parser.get_token()?;
        if matches!(token.get_type(), TokenType::Lr(')')) {
            break;
        }

        let element = match token.get_type() {
            TokenType::Identifier => {
                let next = parser.get_token()?;
                if matches!(next.get_type(), TokenType::Operator(OperatorEnum::Set)) {
                    let value_token = parser.get_token()?;
                    AnnotationElement::Arg {
                        key: token,
                        value: parse_meta_value(value_token)?,
                    }
                } else {
                    parser.cache = Some(next);
                    AnnotationElement::Positional(token)
                }
            }
            TokenType::String(_)
            | TokenType::Char(_)
            | TokenType::Number(_)
            | TokenType::True
            | TokenType::False
            | TokenType::Null => AnnotationElement::Positional(token),
            _ => return Err(ParserError::ExpectedToken(token, TokenType::Identifier)),
        };
        elements.push(element);

        let sep = parser.get_token()?;
        match sep.get_type() {
            TokenType::Operator(OperatorEnum::Comma) => continue,
            TokenType::Lr(')') => break,
            _ => return Err(ParserError::Expected(sep, ')')),
        }
    }

    Ok(elements)
}

pub fn annotation_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let token = parser.get_token()?;
    let TokenType::Identifier = token.get_type() else {
        return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
    };
    let name = token;
    let elements = element_parser(parser)?;
    Ok(AstNode::Annotation { name, elements })
}
