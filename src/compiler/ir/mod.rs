use super::lexer::OperatorEnum;
use crate::compiler::lexer::Token;
use crate::compiler::SourceFile;

#[derive(Debug, Clone)]
pub enum GuardNode {
    Expr(ExprNode),
    Let { head: Pattern, vars: ExprNode },
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard, // _
    Variant {
        name: Token,
        args: Vec<Pattern>,
    },
    Bind(Token),
    Literal(Token),
    Or(Vec<Pattern>),    // a | b | c
    Tuple(Vec<Pattern>), // (a, b)
    Constructor {
        // path::path(args)
        path: Vec<Token>,
        args: Vec<Pattern>,
    },
}

#[derive(Debug, Clone)]
pub enum ExprNode {
    Literal(Token),
    Identifier(Token),
    Try(Box<ExprNode>), // ?
    Unpack(Box<ExprNode>), // !
    Binary {
        token: Token,
        operator: OperatorEnum,
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },
    Unary {
        token: Token,
        operator: OperatorEnum,
        child: Box<ExprNode>,
    },
    CallCort {
        // 枚举 / 函数调用
        call: Box<ExprNode>,
        args: Vec<ExprNode>,
    },
    Cons {
        // 三元表达式
        cons: Box<ExprNode>,
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },
    Closure {
        // 闭包
        args: Vec<AstNode>,
        ret: Token,
        blk: AstNode,
    },
    IfPattern {
        branches: Vec<GuardNode>,
        body: AstNode,
        else_body: Option<AstNode>,
    },
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Import {
        name: Token,
        file: Box<SourceFile>,
    },
    Define {
        name: Token,
        type_name: Option<Token>,
    },
    DefineElse {
        head: Pattern,
        type_name: Option<Token>,
        vars: Option<Box<ExprNode>>,
        el_blk: Option<Box<AstNode>>,
    },
    ForPattern {
        pattern: Pattern,
        exit: bool, // exit ? break : continue
        iter: Box<ExprNode>,
        blk: Box<AstNode>,
    },
    WhilePattern {
        patterns: Vec<GuardNode>,
        body: Box<AstNode>,
    },
    While {
        cond: Box<ExprNode>,
        body: Box<AstNode>,
    },
    Loop {
        body: Box<AstNode>,
    },
    Block {
        body: Vec<AstNode>,
        tail: Option<Box<ExprNode>>
    },
    Function {
        name: Token,
        generics: Option<Vec<Token>>,
        ret_type: Token,
        args: Vec<AstNode>,
        block: Box<AstNode>,
    },
    Expr(Box<ExprNode>),
    EnumDefine {
        name: Token,
        variants: Vec<(Token, Vec<Token>)>,
    },
    StructDefine {
        name: Token,
        generics: Option<Vec<Token>>,
        fields: Vec<(Token, Token)>,
    },
}
