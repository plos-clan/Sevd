use crate::compiler::SourceFile;
use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hir::{FieldId, ItemId, MemberId};
use crate::compiler::lexer::Token;
use crate::compiler::sematic::SemanticLevel;
use crate::compiler::symtbl::{Item, ItemType};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId(pub usize);

#[derive(Debug, Clone)]
pub struct Module {
    pub id: ModuleId,
    pub name: String,
    pub file: Option<SourceFile>,
    pub level: SemanticLevel,
    pub modules: Vec<Module>,

    pub items: Vec<(ItemId, Item)>,
    pub members: Vec<(MemberId, Token)>,
    item_id_alloc: usize,
    member_id_alloc: usize,
}

impl Module {
    #[must_use]
    pub fn new(id: ModuleId, name: String, file: Option<SourceFile>) -> Self {
        Self {
            id,
            name,
            level: SemanticLevel::Load,
            modules: Vec::new(),
            file,
            items: vec![],
            members: vec![],
            item_id_alloc: 0,
            member_id_alloc: 0,
        }
    }

    pub fn alloc_item_id(&mut self) -> ItemId {
        let id = ItemId(self.item_id_alloc);
        self.item_id_alloc += 1;
        id
    }

    pub fn alloc_member_id(&mut self) -> MemberId {
        let id = MemberId(self.member_id_alloc);
        self.member_id_alloc += 1;
        id
    }

    /// # Errors
    ///
    /// check multiple define
    pub fn collect_item_define(
        &mut self,
        id: ItemId,
        name: Token,
        types: ItemType,
    ) -> Result<(), SematicError> {
        if self
            .items
            .iter()
            .any(|(_, existing)| existing.name.get_span().text() == name.get_span().text())
        {
            return Err(SematicError::ItemAlreadyDefined(name));
        }
        self.items.push((id, Item::new(name, types)));
        Ok(())
    }

    /// # Errors
    ///
    /// check multiple define
    pub fn collect_member_define(&mut self, id: MemberId, name: Token) -> Result<(), SematicError> {
        if self
            .members
            .iter()
            .any(|(_, existing)| existing.get_span().text() == name.get_span().text())
        {
            return Err(SematicError::ItemAlreadyDefined(name));
        }
        self.members.push((id, name));
        Ok(())
    }

    /// # Errors
    ///
    /// check field multiple define
    pub fn collect_item_field(&mut self, id: ItemId, name: Token) -> Result<FieldId, SematicError> {
        let Some(item) = self.find_item_mut_by_id(id) else {
            unreachable!()
        };
        item.add_field(name)
    }

    #[must_use]
    pub fn find(&self, parts: &[String]) -> Option<&Module> {
        let Some((name, rest)) = parts.split_first() else {
            return Some(self);
        };
        self.modules
            .iter()
            .find(|module| module.name == *name)?
            .find(rest)
    }

    #[must_use]
    pub fn find_by_id(&self, id: ModuleId) -> Option<&Module> {
        if self.id == id {
            return Some(self);
        }
        self.modules.iter().find_map(|module| module.find_by_id(id))
    }

    pub fn find_by_id_mut(&mut self, id: ModuleId) -> Option<&mut Module> {
        if self.id == id {
            return Some(self);
        }
        self.modules
            .iter_mut()
            .find_map(|module| module.find_by_id_mut(id))
    }

    #[must_use]
    pub fn get_module_path(&self, target: &Module) -> Option<Vec<String>> {
        self.get_module_path_by_id(target.id)
    }

    #[must_use]
    pub fn get_module_path_by_id(&self, id: ModuleId) -> Option<Vec<String>> {
        if self.id == id {
            return Some(vec![self.name.clone()]);
        }
        for module in &self.modules {
            if let Some(mut path) = module.get_module_path_by_id(id) {
                path.insert(0, self.name.clone());
                return Some(path);
            }
        }
        None
    }

    pub fn find_item_mut_by_id(&mut self, id: ItemId) -> Option<&mut Item> {
        self.items
            .iter_mut()
            .find_map(|(index, item)| if *index == id { Some(item) } else { None })
    }
}
