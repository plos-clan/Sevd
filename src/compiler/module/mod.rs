use crate::compiler::SourceFile;

#[derive(Debug, Clone)]
pub enum SubModule {
    File(SourceFile),
    SubModule {
        name: String,
        root: Option<SourceFile>,
        mods: Vec<SubModule>,
    },
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub files: Vec<SubModule>,
}

impl Module {
    pub fn new(name: String) -> Self {
        Self {
            name,
            files: Vec::new(),
        }
    }
}
