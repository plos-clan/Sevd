
use crate::compiler::com_error::ParserError;
use crate::compiler::ir::{AstNode, ExprNode};
use crate::compiler::lexer::{OperatorEnum, Token, TokenType};
use crate::compiler::parser::block::block_parser;
use crate::compiler::parser::expr::ExprParser;
use crate::compiler::parser::Parser;
use crate::compiler::parser::pattern::pattern_parser;

pub fn get_collect_expr(
    tokens: Vec<Token>,
    last_token: &Token,
) -> Result<ExprNode, ParserError> {
    let mut parser = Parser::new_collect(tokens);
    let mut exprs = ExprParser::new(&mut parser, last_token.clone());
    exprs.parse()
}

fn get_expr(parser: &mut Parser) -> Result<ExprNode, ParserError> {
    let mut tokens = vec![];
    let mut token;
    loop {
        token = parser.get_token()?;
        if let TokenType::Operator(OperatorEnum::And) = token.get_type() {
            let and = token;
            token = parser.get_token()?;
            if let TokenType::Let = and.get_type() {
                break;
            }
            tokens.push(and);
        }
        tokens.push(token);
    }
    get_collect_expr(tokens, &token)
}

fn parser_let(parser: &mut Parser) -> Result<Option<AstNode>, ParserError> {
    let mut token = parser.get_token()?;

    if let TokenType::Lp('{') = token.get_type() {
        parser.cache = Some(token);
        return Ok(None);
    }

    if let TokenType::Let = token.get_type() {
        let head = pattern_parser(parser)?;
        token = parser.get_token()?;
        let TokenType::Operator(OperatorEnum::Set) = token.get_type() else {
            return Err(ParserError::Expected(token, '='));
        };
        let expr = get_expr(parser)?;
        return  Ok(Some(AstNode::DefineElse {
            head,
            type_name: None,
            vars: Some(expr),
            el_blk: None,
        }));
    }

    Ok(Some(AstNode::Expr(get_expr(parser)?)))
}

pub fn while_parser(parser: &mut Parser) -> Result<AstNode, ParserError> {
    let token = parser.get_token()?;

    match token.get_type() {
        TokenType::Lp('{') => {
            parser.cache = Some(token);
            let body = block_parser(parser)?;
            Ok(AstNode::Loop {
                body
            })
        },
        _=> {
            let mut patterns: Vec<AstNode> = vec![];
            while let Some(node) = parser_let(parser)? {
                patterns.push(node);
            }
            let body = block_parser(parser)?;
            Ok(AstNode::WhilePattern {
                patterns,
                body
            })
        }
    }
}
