pub mod hast;
pub mod hir;
pub mod refast;
pub mod tnode;

use super::lexer::OperatorEnum;
use crate::compiler::lexer::Token;

#[derive(Debug, Clone)]
pub enum MetaValue {
    Identifier(Token),
    Literal(Token),
    List(Vec<AnnotationElement>),
}

#[derive(Debug, Clone)]
pub enum AnnotationElement {
    Positional(Token),
    Arg { key: Token, value: MetaValue },
}

#[derive(Debug, Clone)]
pub enum GuardNode {
    Expr(ExprNode),
    Let { head: Pattern, vars: ExprNode },
}

#[derive(Debug, Clone)]
pub enum TypeRef {
    Tuple(Vec<TypeRef>),
    Named { name: Token, generics: Vec<TypeRef> },
    Ident(Token),
    Understood, // 等待推导的类型
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard(Token), // _
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

impl Pattern {
    pub fn first_token(&self) -> Option<&Token> {
        match self {
            Self::Bind(token) | Self::Literal(token) | Self::Wildcard(token) => Some(token),

            Self::Variant { name, .. } => Some(name),

            Self::Constructor { path, .. } => path.first(),

            Self::Or(patterns) | Self::Tuple(patterns) => {
                patterns.iter().find_map(Self::first_token)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprNode {
    Error, // 语义分析错误占位节点
    Literal(Token),
    Identifier {
        ident: Token,
        generics: Option<Vec<TypeRef>>,
    },
    Try(Box<ExprNode>),    // ?
    Unpack(Box<ExprNode>), // !
    Tuple(Vec<ExprNode>),
    As {
        token: Token,
        src: Box<ExprNode>,
        target: TypeRef,
    },
    Struct {
        name: Token,
        fields: Vec<(Token, ExprNode)>,
        generics: Option<Vec<TypeRef>>,
    },
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
        generics: Option<Vec<TypeRef>>,
    },
    Closure {
        // 闭包
        args: Vec<AstNode>,
        ret: TypeRef,
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
        name: Vec<Token>,
        path: Token,
    },
    Annotation {
        name: Token,
        elements: Vec<AnnotationElement>,
    },
    Define {
        name: Token,
        type_name: Option<TypeRef>,
    },
    DefineElse {
        token: Token,
        head: Pattern,
        type_name: Option<TypeRef>,
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
        tail: Option<Box<AstNode>>,
    },
    Function {
        name: Token,
        generics: Option<Vec<Token>>,
        constraint: Option<Vec<(Token, Vec<TypeRef>)>>,
        ret_type: TypeRef,
        args: Vec<AstNode>,
        block: Box<AstNode>,
    },
    Expr(Box<ExprNode>),
    Return(Box<ExprNode>),
    EnumDefine {
        name: Token,
        variants: Vec<(Token, Vec<TypeRef>)>,
    },
    StructDefine {
        name: Token,
        generics: Option<Vec<Token>>,
        fields: Vec<(Token, TypeRef)>,
    },
}
