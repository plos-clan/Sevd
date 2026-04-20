use crate::compiler::lexer::Token;
use crate::compiler::SourceFile;
use super::lexer::OperatorEnum;

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
        blk: Vec<AstNode>,
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
        vars: Option<ExprNode>,
    },
    DefineElse {
        head: Pattern,
        type_name: Option<Token>,
        vars: Option<ExprNode>,
        el_blk: Option<Vec<AstNode>>,
    },
    For {
        element: Token,
        iter: ExprNode,
    },
    ForPattern {
        element: Pattern,
        exit: bool, // exit ? break : continue
        iter: ExprNode,
    },
    Function {
        name: Token,
        ret_type: Token,
        args: Vec<AstNode>,
        block: Vec<AstNode>,
    },
    Expr(ExprNode),
    EnumDefine {
        name: Token,
        variants: Vec<(Token, Vec<Token>)>,
    },
}
