/// 用于 function/enum/struct/全局let 的 SymbolID
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(pub usize);

/// 用于 局部变量/形参/模式绑定 的 SymbolID
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalId(pub usize);

/// 用于 枚举和结构体成员 的 SymbolID
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldId(pub usize);
