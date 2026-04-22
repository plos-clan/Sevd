use super::{AnnotationElement, AstNode, ExprNode, GenericArg, Pattern};
use crate::compiler::lexer::Token;

#[derive(Debug, Clone)]
pub struct HastGlobal {
    pub pat: Pattern,
    pub ty: GenericArg,
    pub attributes: Vec<(Token, Vec<AnnotationElement>)>,
    pub init: ExprNode,
    pub el: Option<AstNode>,
}

#[derive(Debug, Clone)]
pub struct HastFunction {
    pub args: Vec<(Token, GenericArg)>,
    pub generics: Option<Vec<Token>>,
    pub ret: GenericArg,
    pub name: Token,
    pub attributes: Vec<(Token, Vec<AnnotationElement>)>,
    pub body: AstNode,
}

#[derive(Debug, Clone)]
pub struct HastStruct {
    pub name: Token,
    pub generics: Option<Vec<Token>>,
    pub fields: Vec<(Token, GenericArg)>,
    pub attributes: Vec<(Token, Vec<AnnotationElement>)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HastEnum {
    pub(crate) name: Token,
    pub(crate) variants: Vec<(Token, Vec<GenericArg>)>,
    pub(crate) attributes: Vec<(Token, Vec<AnnotationElement>)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum HastItem {
    Function(Box<HastFunction>),
    Global(Box<HastGlobal>),
    Struct(HastStruct),
    Enum(HastEnum),
}
