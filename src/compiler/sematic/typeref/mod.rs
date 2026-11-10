mod import;
mod items;

use crate::compiler::ir::hast::HastItem;
use crate::compiler::sematic::Semantic;
use crate::compiler::sematic::typeref::import::collect_import;
use crate::compiler::sematic::typeref::items::ref_struct;

pub fn resolver(semantic: &mut Semantic, items: Vec<HastItem>) {
    let imports = collect_import(semantic, items.as_slice());
    let mut asts = vec![];
    for item in items {
        match item {
            HastItem::Error | HastItem::Import { .. } => {}

            HastItem::Function(_, _) => {}
            HastItem::Global(_) => {}
            HastItem::Struct(id, stc) => asts.push(ref_struct(semantic, id, stc, &imports)),
            HastItem::Enum(id, enm) => {}
        }
    }
}
