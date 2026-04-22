use crate::compiler::com_error::ParserError;
use crate::compiler::com_error::ParserError::{IllegalExpression, IllegalKey};
use crate::compiler::ir::{AstNode, ExprNode};
use crate::compiler::lexer::{OperatorEnum, Token, TokenType};
use crate::compiler::parser::block::block_parser;
use crate::compiler::parser::ifs::if_parser;
use crate::compiler::parser::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ExprType {
    Cond,
    Init,
}

pub struct ExprParser<'a, 'b> {
    fallback_token: Token,
    parser: &'a mut Parser<'b>,
    types: ExprType,
}

impl<'a, 'b> ExprParser<'a, 'b> {
    pub fn new(parser: &'a mut Parser<'b>, fallback_token: Token, types: ExprType) -> ExprParser<'a, 'b> {
        Self {
            fallback_token,
            parser,
            types,
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
            TokenType::Operator(OperatorEnum::Question | OperatorEnum::Not) => Some((26, ())),
            TokenType::Lp('[' | '(' | '{') => Some((27, ())),
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

    fn field_parser(&mut self, depth: usize) -> Result<Vec<(Token, ExprNode)>, ParserError> {
        let mut fields = Vec::new();
        loop {
            let mut token = self.get_token()?;
            if matches!(token.get_type(), TokenType::Lr('}')) {
                break;
            }
            if !matches!(token.get_type(), TokenType::Identifier) {
                return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
            }
            let name = token;
            token = self.get_token()?;
            if !matches!(token.get_type(), TokenType::Operator(OperatorEnum::Colon)) {
                return Err(ParserError::Expected(token, ':'));
            }
            let node = self.expr_bp(0, depth + 1)?;
            fields.push((name, node));
            token = self.get_token()?;
            match token.get_type() {
                TokenType::Operator(OperatorEnum::Comma) => continue,
                TokenType::Lr('}') => break,
                _ => return Err(ParserError::Expected(token, '}')),
            }
        }

        Ok(fields)
    }

    fn parse_ident(&mut self, ident: Token, depth: usize) -> Result<ExprNode, ParserError> {
        if depth == 0 && self.types == ExprType::Cond {
            return Ok(ExprNode::Identifier(ident));
        }
        let token = self.get_token()?;
        let TokenType::Lp('{') = token.get_type() else {
            self.parser.cache = Some(token);
            return Ok(ExprNode::Identifier(ident));
        };

        let fields = self.field_parser(depth)?;

        Ok(ExprNode::Struct {
            name: ident,
            fields,
        })
    }

    fn parse_head(&mut self, token: Token, depth: usize) -> Result<ExprNode, ParserError> {
        match token.get_type() {
            TokenType::Lp(c) if c == &'(' => {
                let next_token = self.get_token()?;
                if let TokenType::Lr(')') = next_token.get_type() {
                    return Ok(ExprNode::Tuple(vec![]));
                }
                self.parser.cache = Some(next_token);
                let lhs = self.expr_bp(0, depth + 1)?;
                let mut n_token = self
                    .get_token()
                    .map_err(|_| ParserError::MissingCondition(token.clone()))?;

                match n_token.get_type() {
                    TokenType::Lr(')') => Ok(lhs),
                    TokenType::Operator(OperatorEnum::Comma) => {
                        let mut fileds = vec![lhs];
                        loop {
                            let filed = self.expr_bp(0, depth + 1)?;
                            fileds.push(filed);
                            n_token = self.get_token()?;
                            match n_token.get_type() {
                                TokenType::Lr(')') => break,
                                TokenType::Operator(OperatorEnum::Comma) => continue,
                                _ => return Err(ParserError::Expected(n_token, ')')),
                            }
                        }
                        Ok(ExprNode::Tuple(fileds))
                    }
                    _ => Err(ParserError::Expected(token, ')')),
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
                let child = self.expr_bp(r_bp, depth + 1)?;
                Ok(ExprNode::Unary {
                    token: token.clone(),
                    operator: *operator,
                    child: Box::new(child),
                })
            }
            TokenType::If => Ok(if_parser(self.parser)?),
            TokenType::Identifier => self.parse_ident(token, depth),
            TokenType::String(_)
            | TokenType::Number(_)
            | TokenType::True
            | TokenType::False
            | TokenType::Null => Ok(ExprNode::Literal(token)),
            _ => Err(IllegalKey(token)),
        }
    }

    fn parser_arguments(&mut self, depth: usize) -> Result<Vec<ExprNode>, ParserError> {
        let mut arguments: Vec<ExprNode> = vec![];
        loop {
            let mut token = self.get_token()?;
            if let TokenType::Lr(')') = token.get_type() {
                break;
            }
            self.parser.cache = Some(token);
            let expr = self.expr_bp(0, depth + 1)?;
            arguments.push(expr);
            token = self.get_token()?;
            match token.get_type() {
                TokenType::Operator(OperatorEnum::Comma) => continue,
                TokenType::Lr(')') => break,
                _ => return Err(ParserError::Expected(token, ',')),
            }
        }
        Ok(arguments)
    }

    fn check_operator(token: &Token) -> bool {
        matches!(
            token.get_type(),
            TokenType::Operator(_) | TokenType::Lp('[' | '(')
        )
    }

    fn expr_bp(&mut self, min_bp: u8, depth: usize) -> Result<ExprNode, ParserError> {
        let token = self.get_token()?;
        let mut expr_tree = self.parse_head(token, depth)?;
        while let Ok(token) = self.get_token() {
            if !Self::check_operator(&token) {
                self.parser.cache = Some(token);
                break;
            }

            if let Some((l_bp, ())) = Self::postfix_binding_power(&token) {
                if l_bp < min_bp {
                    self.parser.cache = Some(token);
                    break;
                }

                match token.get_type() {
                    TokenType::Lp(c) => {
                        match c {
                            '[' => {
                                // 用于解析数组获取值, 如: array[index]
                                let rhs = self.expr_bp(0, depth + 1)?;
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
                                    args: self.parser_arguments(depth)?,
                                    generics: None,
                                }
                            }
                            '|' => {}
                            _ => return Err(ParserError::Expected(token, '(')),
                        }
                    }
                    TokenType::Operator(OperatorEnum::Question) => {
                        expr_tree = ExprNode::Try(Box::new(expr_tree))
                    }

                    TokenType::Operator(OperatorEnum::Not) => {
                        expr_tree = ExprNode::Unpack(Box::new(expr_tree))
                    }

                    TokenType::Operator(operator)
                    if !matches!(operator, OperatorEnum::Plus | OperatorEnum::Minus) =>
                        {
                            expr_tree = ExprNode::Unary {
                                token: token.clone(),
                                operator: *operator,
                                child: Box::new(expr_tree),
                            }
                        }
                    _ => return Err(IllegalExpression(token)),
                }
                continue;
            }

            if let Some((l_bp, r_bp)) = Self::infix_binding_power(&token) {
                if l_bp < min_bp {
                    self.parser.cache = Some(token);
                    break;
                }

                // 二元表达式解析
                let rhs = self.expr_bp(r_bp, depth)?;
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
                    ParserError::ExpectedToken(self.fallback_token.clone(), TokenType::Identifier)
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
            TokenType::Operator(OperatorEnum::BitOr) => Ok(Some((
                AstNode::Define {
                    name,
                    type_name: None,
                },
                true,
            ))),
            TokenType::Operator(OperatorEnum::Comma) => {
                self.parser.cache = Some(token);
                Ok(Some((
                    AstNode::Define {
                        name,
                        type_name: None,
                    },
                    false,
                )))
            }
            TokenType::Operator(OperatorEnum::Colon) => {
                let type_name = self.get_token().map_err(|_| {
                    ParserError::ExpectedToken(token.clone(), TokenType::Identifier)
                })?;
                let TokenType::Identifier = type_name.get_type() else {
                    return Err(ParserError::ExpectedToken(token, TokenType::Identifier));
                };
                self.parser.cache = Some(type_name.clone());
                Ok(Some((
                    AstNode::Define {
                        name,
                        type_name: Some(type_name),
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
        self.expr_bp(0, 0)
    }
}

pub fn get_of_else_end_expr(parser: &mut Parser, last: &Token, types: ExprType) -> Result<ExprNode, ParserError> {
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
    let mut exprs = ExprParser::new(&mut parser, last.clone(), types);
    exprs.parse()
}

pub enum ExprTerminator {
    End,      // ;
    BlockEnd, // }
}

pub fn get_of_end_or_block_end_expr(
    parser: &mut Parser,
    last: &Token,
    types: ExprType,
) -> Result<(ExprNode, ExprTerminator), ParserError> {
    let mut tokens: Vec<Token> = vec![];
    let mut p_count: usize = 0;
    let mut terminator = ExprTerminator::End;
    loop {
        let token = parser.get_token()?;
        match token.get_type() {
            TokenType::End if p_count == 0 => break,
            TokenType::Lp('{' | '(' | '[') => {
                p_count += 1;
                tokens.push(token);
            }
            TokenType::Lr(')' | ']') => {
                if p_count == 0 {
                    return Err(IllegalExpression(token));
                }
                p_count -= 1;
                tokens.push(token);
            }
            TokenType::Lr('}') => {
                if p_count == 0 {
                    terminator = ExprTerminator::BlockEnd;
                    break;
                }
                p_count -= 1;
                tokens.push(token);
            }
            TokenType::Eof => return Err(IllegalExpression(token)),
            _ => tokens.push(token),
        }
    }
    let mut parser = Parser::new_collect(tokens);
    let mut exprs = ExprParser::new(&mut parser, last.clone(), types);
    Ok((exprs.parse()?, terminator))
}
