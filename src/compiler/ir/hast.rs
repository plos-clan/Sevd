use super::{AnnotationElement, AstNode, ExprNode, Pattern, TypeRef};
use crate::compiler::ir::hir::ItemId;
use crate::compiler::lexer::{OperatorEnum, Token};
use crate::compiler::module::ModuleId;

#[derive(Debug, Clone)]
pub enum HastExpr {
    Literal(Token),
    Identifier {
        ident: Token,
        generics: Option<TypeRef>,
    },
    Try(Box<HastExpr>),    // ?
    Unpack(Box<HastExpr>), // !
    Tuple(Vec<HastExpr>),
    Struct {
        name: Token,
        fields: Vec<(Token, HastExpr)>,
        generics: Option<TypeRef>,
    },
    Binary {
        token: Token,
        operator: OperatorEnum,
        ty: TypeRef,
        left: Box<HastExpr>,
        right: Box<HastExpr>,
    },
    Unary {
        token: Token,
        ty: TypeRef,
        operator: OperatorEnum,
        child: Box<HastExpr>,
    },
    CallCort {
        // 枚举 / 函数调用
        call: Box<HastExpr>,
        args: Vec<HastExpr>,
        generics: Option<Vec<TypeRef>>,
    },
    Closure {
        // 闭包
        args: Vec<AstNode>,
        ret: TypeRef,
        blk: AstNode,
    },
    IfPattern {
        cond: Vec<HastExpr>,
        body: HastStmt,
        else_body: Option<HastStmt>,
    },
}

#[derive(Debug, Clone)]
pub enum HastStmt {
    Local {
        name: Token,
        ty: TypeRef,
        init: Option<Box<HastExpr>>,
    },
    Block {
        body: Vec<HastStmt>,
        tail: Option<Box<HastExpr>>,
    },
}

#[derive(Debug, Clone)]
pub struct HastGlobal {
    pub pat: Pattern,
    pub ty: TypeRef,
    pub attributes: Vec<(Token, Vec<AnnotationElement>)>,
    pub init: ExprNode,
    pub el: Option<AstNode>,
}

#[derive(Debug, Clone)]
pub struct HastFunction {
    pub args: Vec<(Token, TypeRef)>,
    pub constraint: Option<Vec<(Token, Vec<TypeRef>)>>,
    pub ret: TypeRef,
    pub name: Token,
    pub attributes: Vec<(Token, Vec<AnnotationElement>)>,
    pub body: AstNode,
}

#[derive(Debug, Clone)]
pub struct HastStruct {
    pub name: Token,
    pub generics: Option<Vec<Token>>,
    pub fields: Vec<(Token, TypeRef)>,
    pub attributes: Vec<(Token, Vec<AnnotationElement>)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HastEnum {
    pub(crate) name: Token,
    pub(crate) variants: Vec<(Token, Vec<TypeRef>)>,
    pub(crate) attributes: Vec<(Token, Vec<AnnotationElement>)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum HastItem {
    Import {
        name: Vec<Token>,
        module_id: ModuleId,
    },
    Function(Box<HastFunction>),
    Global(Box<HastGlobal>),
    Struct(ItemId, HastStruct),
    Enum(HastEnum),
}
