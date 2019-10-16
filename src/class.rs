use crate::node::Node;
use crate::util::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ClassInfo {
    pub id: IdentId,
    pub name: String,
    pub body: Box<Node>,
    pub subclass: HashMap<IdentId, ClassRef>,
}

impl ClassInfo {
    pub fn new(id: IdentId, name: String, body: Node) -> Self {
        ClassInfo {
            id,
            name,
            body: Box::new(body),
            subclass: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassRef(usize);

impl std::hash::Hash for ClassRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalClassTable {
    table: HashMap<ClassRef, ClassInfo>,
    class_id: usize,
}

impl GlobalClassTable {
    pub fn new() -> Self {
        GlobalClassTable {
            table: HashMap::new(),
            class_id: 0,
        }
    }

    pub fn new_class(&mut self, id: IdentId, name: String, body: Node) -> ClassRef {
        let info = ClassInfo::new(id, name, body);
        let new_class = ClassRef(self.class_id);
        self.class_id += 1;
        self.table.insert(new_class, info);
        new_class
    }

    pub fn get(&mut self, class_ref: ClassRef) -> &ClassInfo {
        self.table
            .get(&class_ref)
            .unwrap_or_else(|| panic!("GlobalClassTable#get(): ClassRef is not valid."))
    }

    pub fn get_mut(&mut self, class_ref: ClassRef) -> &mut ClassInfo {
        self.table
            .get_mut(&class_ref)
            .unwrap_or_else(|| panic!("GlobalClassTable#get_mut(): ClassRef is not valid."))
    }
}
