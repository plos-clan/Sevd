use crate::compiler::ir::hir::{FieldId, ItemId, MemberId, TypeVarId};
use crate::compiler::ir::tnode::PrimitiveType;
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
}

pub struct TypeKind {
    pub mod_id: ModuleId,
    pub meb_id: MemberId,
}

#[derive(Debug, Clone)]
pub enum RefAstType {
    Primitive(PrimitiveType), // base type
    Tuple(Vec<RefAstType>),   // (i32, i64)
    Single(TypeDefId),        // 单一的类型(不带泛型参数)
    Adt {
        def: TypeDefId,
        args: Vec<RefAstType>,
    }, // struct!<args>/enum(args)
    Infer(TypeVarId),         // <T> 泛型回填
    Understood(TypeVarId),    // 推导回填
}

pub struct RefStruct {
    pub fields: Vec<(FieldId, RefAstType)>,
}

pub enum RefAst {
    Struct(ItemId, RefStruct),
}

impl ImportedName {
    #[must_use]
    pub fn get_token(&self) -> &Token {
        match self {
            ImportedName::TypeId(token, _)
            | ImportedName::MemberId(token, _)
            | ImportedName::Double(token, _, _) => token,
        }
    }
}
