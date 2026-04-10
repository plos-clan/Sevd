use crate::compiler::com_error::ParserError;
use crate::compiler::com_error::ParserError::{IllegalExpression, IllegalKey};
use crate::compiler::ir::{AstNode, ExprNode};
use crate::compiler::lexer::{OperatorEnum, Token, TokenType};
use crate::compiler::parser::Parser;
use crate::compiler::parser::block::block_parser;
use std::iter::Peekable;
use std::vec::IntoIter;

struct ExprParser {
    fallback_token: Token,
    tokens: Peekable<IntoIter<Token>>,
}

impl ExprParser {
    fn new(tokens: Vec<Token>, fallback_token: Token) -> ExprParser {
        Self {
            fallback_token,
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn check_operator(token: &Token) -> Result<(), ParserError> {
        match token.get_type() {
            TokenType::Lr('}' | ']' | ')') => Ok(()),
            TokenType::Operator(_) => Ok(()),
            _ => Err(IllegalExpression(token.clone())),
        }
    }

    fn prefix_binding_power(operator: &OperatorEnum) -> ((), u8) {
        match operator {
            OperatorEnum::Plus | OperatorEnum::Minus => ((), 21),
            OperatorEnum::Not | OperatorEnum::Add | OperatorEnum::Sub => ((), 23),
            _ => ((), 0),
        }
    }

    fn postfix_binding_power(token: &Token) -> Option<(u8, ())> {
        match token.get_type() {
            TokenType::Operator(OperatorEnum::Plus | OperatorEnum::Minus) => Some((21, ())),
            TokenType::Lp('[' | '(') | TokenType::Operator(OperatorEnum::BitOr) => Some((27, ())),
            _ => None,
        }
    }

    fn infix_binding_power(token: &Token) -> Option<(u8, u8)> {
        if let TokenType::Operator(operator) = token.get_type() {
            return match operator {
                OperatorEnum::Set | OperatorEnum::AddSet | OperatorEnum::SubSet => Some((2, 1)),
                OperatorEnum::Question => Some((4, 3)),
                OperatorEnum::And | OperatorEnum::Or => Some((5, 6)),
                OperatorEnum::BitOr => Some((7, 8)),
                OperatorEnum::BitXor => Some((9, 10)),
                OperatorEnum::BitAnd => Some((11, 12)),
                OperatorEnum::Eq | OperatorEnum::NotEq => Some((13, 14)),
                OperatorEnum::Ref => Some((30, 29)),
                _ => None,
            };
        }
        None
    }

    fn parse_head(&mut self, token: Token) -> Result<ExprNode, ParserError> {
        match token.get_type() {
            TokenType::Lp(c) if c == &'(' => {
                let lhs = self.expr_bp(0)?;
                let Some(n_token) = self.tokens.next() else {
                    return Err(ParserError::MissingCondition(token));
                };
                if let TokenType::Lr(c) = n_token.get_type()
                    && c == &')'
                {
                    Ok(lhs)
                } else {
                    Err(ParserError::Expected(token, ')'))
                }
            }
            TokenType::Operator(operator) => {
                let ((), r_bp) = Self::prefix_binding_power(operator);
                if let TokenType::Operator(OperatorEnum::BitOr) = token.get_type() {
                    return self.closure_parser(false);
                }
                if let TokenType::Operator(OperatorEnum::Or) = token.get_type() {
                    return self.closure_parser(true);
                }
                if !matches!(
                    operator,
                    OperatorEnum::Add
                        | OperatorEnum::Sub
                        | OperatorEnum::Plus
                        | OperatorEnum::Minus
                        | OperatorEnum::Not
                ) {
                    return Err(IllegalExpression(token));
                }
                self.fallback_token = token.clone();
                let child = self.expr_bp(r_bp)?;
                Ok(ExprNode::Unary {
                    token: token.clone(),
                    operator: *operator,
                    child: Box::new(child),
                })
            }
            TokenType::Identifier => Ok(ExprNode::Identifier(token)),
            TokenType::String(_)
            | TokenType::Number(_)
            | TokenType::True
            | TokenType::False
            | TokenType::Null => Ok(ExprNode::Literal(token)),
            _ => Err(IllegalKey(token)),
        }
    }

    fn parser_arguments(&mut self, mut token: Token) -> Result<Vec<ExprNode>, ParserError> {
        let mut arguments: Vec<ExprNode> = vec![];
        loop {
            let mut sub_tokens: Vec<Token> = vec![];
            token = self
                .tokens
                .next()
                .ok_or(ParserError::MissingCondition(token))?;
            let mut p_count: u64 = 0;
            let mut done: bool = false;
            loop {
                if let TokenType::Operator(OperatorEnum::Comma) = token.get_type()
                    && p_count == 0
                {
                    break;
                }
                if let TokenType::Lp('(' | '{' | '[') = token.get_type() {
                    p_count += 1;
                }
                if let TokenType::Lr('}' | ']') = token.get_type() {
                    p_count -= 1;
                }
                if let TokenType::Lr(')') = token.get_type() {
                    if p_count == 0 {
                        done = true;
                        break;
                    }
                    p_count -= 1;
                }

                sub_tokens.push(token.clone());
                token = self
                    .tokens
                    .next()
                    .ok_or(ParserError::MissingCondition(token))?;
            }
            if let Some(expr) = get_collect_expr(sub_tokens, &token)? {
                arguments.push(expr);
            }
            if done {
                break;
            }
        }
        Ok(arguments)
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<ExprNode, ParserError> {
        let Some(token) = self.tokens.next() else {
            return Err(IllegalExpression(self.fallback_token.clone()));
        };
        let mut expr_tree = self.parse_head(token)?;
        while let Some(token) = self.tokens.peek().cloned() {
            Self::check_operator(&token)?;

            if let Some((l_bp, ())) = Self::postfix_binding_power(&token) {
                if l_bp < min_bp {
                    break;
                }
                self.tokens.next();

                if let TokenType::Lp(c) = token.get_type() {
                    match c {
                        '[' => {
                            // 用于解析数组获取值, 如: array[index]
                            let rhs = self.expr_bp(0)?;
                            match self.tokens.next() {
                                Some(tk) => {
                                    let TokenType::Lr(']') = tk.get_type() else {
                                        return Err(ParserError::Expected(token, ']'));
                                    };
                                    expr_tree = ExprNode::Binary {
                                        token,
                                        operator: OperatorEnum::Array,
                                        left: Box::new(expr_tree),
                                        right: Box::new(rhs),
                                    };
                                }
                                None => {
                                    return Err(ParserError::MissingCondition(token));
                                }
                            }
                        }
                        '(' => {
                            // 用于解析函数调用, 如: call_test(args)
                            expr_tree = ExprNode::CallCort {
                                call: Box::new(expr_tree),
                                args: self.parser_arguments(token)?,
                            }
                        }
                        '|' => {}
                        _ => return Err(ParserError::Expected(token, '(')),
                    }
                } else if let TokenType::Operator(operator) = token.get_type()
                    && !matches!(operator, OperatorEnum::Plus | OperatorEnum::Minus)
                {
                    expr_tree = ExprNode::Unary {
                        token: token.clone(),
                        operator: *operator,
                        child: Box::new(expr_tree),
                    }
                } else {
                    return Err(IllegalExpression(token));
                }
                continue;
            }

            if let Some((l_bp, r_bp)) = Self::infix_binding_power(&token) {
                if l_bp < min_bp {
                    break;
                }
                self.tokens.next();
                // 三元表达式解析
                if let TokenType::Operator(OperatorEnum::Question) = token.get_type() {
                    let mhs = self.expr_bp(0)?;
                    if let Some(colon) = self.tokens.peek().cloned()
                        && let TokenType::Operator(OperatorEnum::Colon) = colon.get_type()
                    {
                        self.tokens.next();
                        let rhs = self.expr_bp(r_bp)?;
                        expr_tree = ExprNode::Cons {
                            cons: Box::new(expr_tree),
                            left: Box::new(mhs),
                            right: Box::new(rhs),
                        };
                        continue;
                    }
                    return Err(ParserError::Expected(token, ':'));
                }

                // 二元表达式解析
                let rhs = self.expr_bp(r_bp)?;
                if let TokenType::Operator(operator) = token.get_type() {
                    expr_tree = ExprNode::Binary {
                        token: token.clone(),
                        operator: *operator,
                        left: Box::new(expr_tree),
                        right: Box::new(rhs),
                    };
                    continue;
                }
                return Err(IllegalExpression(token));
            }
            break;
        }
        Ok(expr_tree)
    }

    fn closure_argument(&mut self) -> Result<Option<(AstNode, bool)>, ParserError> {
        let Some(mut token) = self.tokens.peek().cloned() else {
            return Err(ParserError::Expected(self.fallback_token.clone(), '|'));
        };
        if let TokenType::Operator(OperatorEnum::BitOr) = token.get_type() {
            return Ok(None);
        }

        match token.get_type() {
            TokenType::Identifier => {}
            TokenType::Operator(OperatorEnum::Comma) => {
                // 后参数提取 (,arg2,arg3)
                self.tokens.next();
                let Some(token_opt) = self.tokens.peek().cloned() else {
                    return Err(ParserError::ExpectedToken(
                        self.fallback_token.clone(),
                        TokenType::Identifier,
                    ));
                };
                let TokenType::Identifier = token_opt.get_type() else {
                    return Err(ParserError::ExpectedToken(
                        self.fallback_token.clone(),
                        TokenType::Identifier,
                    ));
                };
                token = token_opt;
            }
            _ => return Err(ParserError::Expected(self.fallback_token.clone(), '|')),
        }

        let name = token;
        self.tokens.next();
        let Some(token) = self.tokens.peek().cloned() else {
            return Err(ParserError::Expected(self.fallback_token.clone(), '|'));
        };
        match token.get_type() {
            TokenType::Operator(OperatorEnum::BitOr) => {
                self.tokens.next();
                Ok(Some((
                    AstNode::Define {
                        name,
                        type_name: None,
                        vars: None,
                    },
                    true,
                )))
            }
            TokenType::Operator(OperatorEnum::Comma) => Ok(Some((
                AstNode::Define {
                    name,
                    type_name: None,
                    vars: None,
                },
                false,
            ))),
            TokenType::Operator(OperatorEnum::Colon) => {
                self.tokens.next();
                let Some(type_name) = self.tokens.peek().cloned() else {
                    return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
                };
                Ok(Some((
                    AstNode::Define {
                        name,
                        type_name: Some(type_name),
                        vars: None,
                    },
                    false,
                )))
            }
            _ => Err(ParserError::Expected(self.fallback_token.clone(), '|')),
        }
    }

    fn closure_parser(&mut self, is_or: bool) -> Result<ExprNode, ParserError> {
        let mut args: Vec<AstNode> = vec![];

        if !is_or {
            while let Some(arg) = self.closure_argument()? {
                args.push(arg.0);
                if arg.1 {
                    break;
                }
            }
        }

        let Some(token) = self.tokens.peek() else {
            return Err(ParserError::Expected(self.fallback_token.clone(), ':'));
        };
        let TokenType::Operator(OperatorEnum::Colon) = token.get_type() else {
            return Err(ParserError::Expected(self.fallback_token.clone(), ':'));
        };

        self.tokens.next();
        let Some(token) = self.tokens.peek().cloned() else {
            return Err(ParserError::ExpectedToken(
                self.fallback_token.clone(),
                TokenType::Identifier,
            ));
        };
        let TokenType::Identifier = token.get_type() else {
            return Err(ParserError::ExpectedToken(
                self.fallback_token.clone(),
                TokenType::Identifier,
            ));
        };
        let ret = token;
        self.tokens.next();

        let Some(token) = self.tokens.peek() else {
            return Err(ParserError::ExpectedToken(ret.clone(), TokenType::From));
        };
        let TokenType::From = token.get_type() else {
            return Err(ParserError::ExpectedToken(ret.clone(), TokenType::From));
        };
        self.fallback_token = token.clone();
        self.tokens.next();

        let Some(token) = self.tokens.next() else {
            return Err(ParserError::Expected(self.fallback_token.clone(), '{'));
        };
        let TokenType::Lp('{') = token.get_type() else {
            return Err(ParserError::Expected(token, '{'));
        };

        let mut blk_tokens: Vec<Token> = vec![token];
        let mut b_count: usize = 1;
        while b_count > 0 {
            let Some(token) = self.tokens.next() else {
                return Err(ParserError::Expected(self.fallback_token.clone(), '}'));
            };
            match token.get_type() {
                TokenType::Lp('{') => b_count += 1,
                TokenType::Lr('}') => b_count -= 1,
                _ => {}
            }
            blk_tokens.push(token);
        }

        let mut parser = Parser::new_collect(blk_tokens);
        Ok(ExprNode::Closure {
            args,
            ret,
            blk: block_parser(&mut parser)?,
        })
    }

    pub fn parse(&mut self) -> Result<ExprNode, ParserError> {
        self.expr_bp(0)
    }
}

pub fn get_collect_expr(
    tokens: Vec<Token>,
    last_token: &Token,
) -> Result<Option<ExprNode>, ParserError> {
    if tokens.is_empty() {
        return Ok(None);
    }
    let mut exprs = ExprParser::new(tokens, last_token.clone());
    Ok(Some(exprs.parse()?))
}

pub fn get_of_else_end_expr(parser: &mut Parser, last: &Token) -> Result<ExprNode, ParserError> {
    let mut tokens: Vec<Token> = vec![];
    let mut p_count: usize = 0;
    loop {
        let token = parser.get_token()?;
        match token.get_type() {
            TokenType::End if p_count == 0 => break,
            TokenType::Else if p_count == 0 => {
                parser.cache = Some(token);
                break;
            }
            TokenType::Lp('{' | '(' | '[') => {
                p_count += 1;
                tokens.push(token);
            }
            TokenType::Lr('}' | ')' | ']') => {
                if p_count == 0 {
                    return Err(IllegalExpression(token));
                }
                p_count -= 1;
                tokens.push(token);
            }
            _ => tokens.push(token),
        }
    }
    let mut exprs = ExprParser::new(tokens, last.clone());
    exprs.parse()
}

pub fn get_of_end_expr(parser: &mut Parser, last: &Token) -> Result<ExprNode, ParserError> {
    let mut tokens: Vec<Token> = vec![];
    let mut p_count: usize = 0;
    loop {
        let token = parser.get_token()?;
        match token.get_type() {
            TokenType::End if p_count == 0 => break,
            TokenType::Lp('{' | '(' | '[') => {
                p_count += 1;
                tokens.push(token);
            }
            TokenType::Lr('}' | ')' | ']') => {
                if p_count == 0 {
                    return Err(IllegalExpression(token));
                }
                p_count -= 1;
                tokens.push(token);
            }
            _ => tokens.push(token),
        }
    }
    let mut exprs = ExprParser::new(tokens, last.clone());
    exprs.parse()
}
