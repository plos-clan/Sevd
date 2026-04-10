use crate::compiler::com_error::ParserError;
use crate::compiler::com_error::ParserError::{IllegalExpression, IllegalKey};
use crate::compiler::ir::{AstNode, ExprNode};
use crate::compiler::lexer::{OperatorEnum, Token, TokenType};
use crate::compiler::parser::block::block_parser;
use crate::compiler::parser::Parser;

struct ExprParser<'a, 'b> {
    fallback_token: Token,
    parser: &'a mut Parser<'b>,
}

impl<'a, 'b> ExprParser<'a, 'b> {
    fn new(parser: &'a mut Parser<'b>, fallback_token: Token) -> ExprParser<'a, 'b> {
        Self {
            fallback_token,
            parser,
        }
    }

    fn get_token(&mut self) -> Result<Token, ParserError> {
        self.parser.get_token()
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
            TokenType::Lp('[' | '(') => Some((27, ())),
            _ => None,
        }
    }

    fn infix_binding_power(token: &Token) -> Option<(u8, u8)> {
        if let TokenType::Operator(operator) = token.get_type() {
            return match operator {
                OperatorEnum::Set
                | OperatorEnum::AddSet
                | OperatorEnum::SubSet
                | OperatorEnum::MulSet
                | OperatorEnum::DivSet
                | OperatorEnum::ModSet => Some((2, 1)),
                OperatorEnum::Question => Some((4, 3)),
                OperatorEnum::And | OperatorEnum::Or => Some((5, 6)),
                OperatorEnum::BitOr => Some((7, 8)),
                OperatorEnum::BitXor => Some((9, 10)),
                OperatorEnum::BitAnd => Some((11, 12)),
                OperatorEnum::Eq | OperatorEnum::NotEq => Some((13, 14)),
                OperatorEnum::BigEq
                | OperatorEnum::LesEq
                | OperatorEnum::Big
                | OperatorEnum::Less => Some((15, 16)),
                OperatorEnum::BitLeft | OperatorEnum::BitRight => Some((17, 18)),
                OperatorEnum::Add | OperatorEnum::Sub => Some((19, 20)),
                OperatorEnum::Mul | OperatorEnum::Div | OperatorEnum::Mod => Some((24, 25)),
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
                let n_token = self
                    .get_token()
                    .map_err(|_| ParserError::MissingCondition(token.clone()))?;
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

    fn parser_arguments(&mut self) -> Result<Vec<ExprNode>, ParserError> {
        let mut arguments: Vec<ExprNode> = vec![];
        loop {
            let expr = self.expr_bp(0)?;
            arguments.push(expr);
            let token = self.get_token()?;
            match token.get_type() {
                TokenType::Operator(OperatorEnum::Comma) => { continue},
                TokenType::Lr(')') => { break},
                _=> return Err(ParserError::Expected(token, ',')),
            }
        }
        Ok(arguments)
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<ExprNode, ParserError> {
        let token = self.get_token()?;
        let mut expr_tree = self.parse_head(token)?;
        while let Ok(token) = self.get_token() {

            if let Some((l_bp, ())) = Self::postfix_binding_power(&token) {
                if l_bp < min_bp {
                    self.parser.cache = Some(token);
                    break;
                }

                if let TokenType::Lp(c) = token.get_type() {
                    match c {
                        '[' => {
                            // 用于解析数组获取值, 如: array[index]
                            let rhs = self.expr_bp(0)?;
                            let tk = self
                                .get_token()
                                .map_err(|_| ParserError::MissingCondition(token.clone()))?;
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
                        '(' => {
                            // 用于解析函数调用, 如: call_test(args)
                            expr_tree = ExprNode::CallCort {
                                call: Box::new(expr_tree),
                                args: self.parser_arguments()?,
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
                    self.parser.cache = Some(token);
                    break;
                }
                // 三元表达式解析
                if let TokenType::Operator(OperatorEnum::Question) = token.get_type() {
                    let mhs = self.expr_bp(0)?;
                    let colon = self
                        .get_token()
                        .map_err(|_| ParserError::Expected(token.clone(), ':'))?;
                    if let TokenType::Operator(OperatorEnum::Colon) = colon.get_type() {
                        let rhs = self.expr_bp(r_bp)?;
                        expr_tree = ExprNode::Cons {
                            cons: Box::new(expr_tree),
                            left: Box::new(mhs),
                            right: Box::new(rhs),
                        };
                        continue;
                    }
                    self.parser.cache = Some(colon);
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
            self.parser.cache = Some(token);
            break;
        }
        Ok(expr_tree)
    }

    fn closure_argument(&mut self) -> Result<Option<(AstNode, bool)>, ParserError> {
        let mut token = self
            .get_token()
            .map_err(|_| ParserError::Expected(self.fallback_token.clone(), '|'))?;
        if let TokenType::Operator(OperatorEnum::BitOr) = token.get_type() {
            self.parser.cache = Some(token);
            return Ok(None);
        }

        match token.get_type() {
            TokenType::Identifier => {}
            TokenType::Operator(OperatorEnum::Comma) => {
                // 后参数提取 (,arg2,arg3)
                let token_opt = self.get_token().map_err(|_| {
                    ParserError::ExpectedToken(
                        self.fallback_token.clone(),
                        TokenType::Identifier,
                    )
                })?;
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
        let token = self
            .get_token()
            .map_err(|_| ParserError::Expected(self.fallback_token.clone(), '|'))?;
        match token.get_type() {
            TokenType::Operator(OperatorEnum::BitOr) => {
                Ok(Some((
                    AstNode::Define {
                        name,
                        type_name: None,
                        vars: None,
                    },
                    true,
                )))
            }
            TokenType::Operator(OperatorEnum::Comma) => {
                self.parser.cache = Some(token);
                Ok(Some((
                    AstNode::Define {
                        name,
                        type_name: None,
                        vars: None,
                    },
                    false,
                )))
            }
            TokenType::Operator(OperatorEnum::Colon) => {
                let type_name = self
                    .get_token()
                    .map_err(|_| ParserError::ExpectedToken(token.clone(), TokenType::Identifier))?;
                let TokenType::Identifier = type_name.get_type() else {
                    return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
                };
                self.parser.cache = Some(type_name.clone());
                Ok(Some((
                    AstNode::Define {
                        name,
                        type_name: Some(type_name),
                        vars: None,
                    },
                    false,
                )))
            }
            _ => {
                self.parser.cache = Some(token);
                Err(ParserError::Expected(self.fallback_token.clone(), '|'))
            }
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

        let token = self
            .get_token()
            .map_err(|_| ParserError::Expected(self.fallback_token.clone(), ':'))?;
        let TokenType::Operator(OperatorEnum::Colon) = token.get_type() else {
            self.parser.cache = Some(token);
            return Err(ParserError::Expected(self.fallback_token.clone(), ':'));
        };

        let token = self.get_token().map_err(|_| {
            ParserError::ExpectedToken(self.fallback_token.clone(), TokenType::Identifier)
        })?;
        let TokenType::Identifier = token.get_type() else {
            self.parser.cache = Some(token);
            return Err(ParserError::ExpectedToken(
                self.fallback_token.clone(),
                TokenType::Identifier,
            ));
        };
        let ret = token;

        let token = self
            .get_token()
            .map_err(|_| ParserError::ExpectedToken(ret.clone(), TokenType::From))?;
        let TokenType::From = token.get_type() else {
            self.parser.cache = Some(token);
            return Err(ParserError::ExpectedToken(ret.clone(), TokenType::From));
        };
        self.fallback_token = token.clone();

        let token = self
            .get_token()
            .map_err(|_| ParserError::Expected(self.fallback_token.clone(), '{'))?;
        let TokenType::Lp('{') = token.get_type() else {
            return Err(ParserError::Expected(token, '{'));
        };

        let mut blk_tokens: Vec<Token> = vec![token];
        let mut b_count: usize = 1;
        while b_count > 0 {
            let token = self
                .get_token()
                .map_err(|_| ParserError::Expected(self.fallback_token.clone(), '}'))?;
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
    let mut parser = Parser::new_collect(tokens);
    let mut exprs = ExprParser::new(&mut parser, last.clone());
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
    let mut parser = Parser::new_collect(tokens);
    let mut exprs = ExprParser::new(&mut parser, last.clone());
    exprs.parse()
}
