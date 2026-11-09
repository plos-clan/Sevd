use crate::compiler::ir::hir::{ItemId, MemberId};
use crate::compiler::lexer::Token;
use crate::compiler::module::ModuleId;

/// 结构体/枚举/基元 类型空间引用id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeDefId {
    pub module: ModuleId,
    pub item: ItemId,
}

/// 函数/全局变量 成员空间引用id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemberDefId {
    pub module: ModuleId,
    pub member: MemberId,
}

#[derive(Debug, Clone)]
pub enum ImportedName {
    TypeId(Token, TypeDefId),
    MemberId(Token, MemberDefId),
    Double(Token, TypeDefId, MemberDefId),
    Error,
}

pub struct TypeKind {
    pub mod_id: ModuleId,
    pub meb_id: MemberId,
}

pub enum RefAst {
    Primitive(TypeDefId),
    Tuple(Vec<TypeKind>),
    Adt { def: TypeDefId, args: Vec<TypeKind> },
}
