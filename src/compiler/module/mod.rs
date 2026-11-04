use crate::compiler::SourceFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub(crate) usize);

#[derive(Debug, Clone)]
pub struct Module {
    pub id: ModuleId,
    pub name: String,
    pub file: Option<SourceFile>,
    pub modules: Vec<Module>,
}

impl Module {
    pub fn new(id: ModuleId, name: String, file: Option<SourceFile>) -> Self {
        Self {
            id,
            name,
            modules: Vec::new(),
            file,
        }
    }

    /// Look up a path relative to this module. An empty path selects this module.
    pub fn find(&self, parts: &[String]) -> Option<&Module> {
        let Some((name, rest)) = parts.split_first() else {
            return Some(self);
        };
        self.modules
            .iter()
            .find(|module| module.name == *name)?
            .find(rest)
    }

    pub fn find_by_id(&self, id: ModuleId) -> Option<&Module> {
        if self.id == id {
            return Some(self);
        }
        self.modules.iter().find_map(|module| module.find_by_id(id))
    }

    pub fn get_module_path(&self, target: &Module) -> Option<Vec<String>> {
        self.get_module_path_by_id(target.id)
    }

    /// Return the path including this root's name, using stable node identity.
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
