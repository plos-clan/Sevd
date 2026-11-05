use crate::compiler::ir::tnode::TypeKind;
use crate::compiler::lexer::{OperatorEnum, Token};
use crate::compiler::module::ModuleId;

/// 用于 enum/struct/union/interface 的类型空间 `SymbolID`
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(pub usize);

/// 用于 function/let 的成员空间 `SymbolID`
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemberId(pub usize);

/// 用于局部变量/形参/模式绑定的 `SymbolID`
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalId(pub usize);

/// 用于枚举和结构体成员的 `SymbolID`
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldId(pub usize);

/// 用于 path1::path2 路径空间的 `SymbolID`
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PathId(pub ModuleId, pub MemberId);

#[derive(Debug, Clone)]
pub enum HirLit {
    UInt(Token, u64),  // unsigned number (u8 - u64)
    IInt(Token, i64),  // signed number (i8 - i64)
    Float(Token, f64), // f32 / f64
    Bool(Token, bool),
    Char(Token, char), // i8
    String(Token, String),
}

#[derive(Debug, Clone)]
pub enum HirExpr {
    Imm(HirLit),
    Ref(Token),

    Unary {
        opt: OperatorEnum,
        child: Box<HirExpr>,
    },
    Binary {
        op: OperatorEnum,
        left: Box<HirExpr>,
        right: Box<HirExpr>,
    },
    /// 分析占位符, 一般用于符号未定义的情况
    Nothing,
}

#[derive(Debug, Clone)]
pub enum HirStmt {
    Expr(HirExpr),
    Blk(Vec<HirStmt>),
}

#[derive(Debug, Clone)]
pub enum HirItem {
    Func { blk: HirStmt, ret: TypeKind },
}
