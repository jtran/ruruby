use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct IvarInfo {
    class_ext: ClassRef,
}

impl IvarInfo {
    pub fn from(class: Module) -> Self {
        Self {
            class_ext: class.ext(),
        }
    }

    pub fn class(&self) -> ClassRef {
        self.class_ext
    }
}
