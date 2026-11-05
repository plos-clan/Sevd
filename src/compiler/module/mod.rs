use crate::compiler::SourceFile;
use crate::compiler::com_error::SematicError;
use crate::compiler::ir::hir::ItemId;
use crate::compiler::lexer::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub(crate) usize);

#[derive(Debug, Clone)]
pub struct Module {
    pub id: ModuleId,
    pub name: String,
    pub file: Option<SourceFile>,
    pub modules: Vec<Module>,
    pub items: Vec<(ItemId, Token)>,
    item_id_alloc: usize,
}

impl Module {
    #[must_use]
    pub fn new(id: ModuleId, name: String, file: Option<SourceFile>) -> Self {
        Self {
            id,
            name,
            modules: Vec::new(),
            file,
            items: vec![],
            item_id_alloc: 0,
        }
    }

    pub fn alloc_item_id(&mut self) -> ItemId {
        let id = ItemId(self.item_id_alloc);
        self.item_id_alloc += 1;
        id
    }

    /// # Errors
    ///
    /// check multiple define
    pub fn collect_item_define(&mut self, id: ItemId, name: Token) -> Result<(), SematicError> {
        if self
            .items
            .iter()
            .any(|(_, existing)| existing.get_span().text() == name.get_span().text())
        {
            return Err(SematicError::ItemAlreadyDefined(name));
        }
        self.items.push((id, name));
        Ok(())
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
}
