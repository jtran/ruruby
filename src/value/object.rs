use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct IvarInfo {
    vec: Vec<Option<Value>>,
    class_ext: ClassRef,
}

impl std::ops::Deref for IvarInfo {
    type Target = Vec<Option<Value>>;
    fn deref(&self) -> &Vec<Option<Value>> {
        &self.vec
    }
}

impl IvarInfo {
    pub fn from(class: Module) -> Self {
        Self {
            vec: vec![],
            class_ext: class.ext(),
        }
    }

    pub fn class(&self) -> ClassRef {
        self.class_ext
    }

    pub fn get_ivar_slot(&mut self, name: IdentId) -> IvarSlot {
        self.class_ext.get_ivar_slot(name)
    }

    pub fn access(&mut self, slot: IvarSlot) -> Value {
        let slot = slot.into_usize();
        self.resize(slot);
        match self.vec[slot] {
            Some(val) => val,
            None => Value::nil(),
        }
    }

    pub fn set(&mut self, slot: IvarSlot, val: Option<Value>) {
        let slot = slot.into_usize();
        self.resize(slot);
        self.vec[slot] = val;
    }

    fn resize(&mut self, slot: usize) {
        if self.vec.len() <= slot {
            let ivar_len = self.class_ext.ivar_len();
            self.vec.resize(ivar_len, None);
        }
    }
}

impl IvarInfo {
    pub fn get(&self, slot: IvarSlot) -> Option<Value> {
        match self.vec.get(slot.into_usize()) {
            Some(Some(val)) => Some(*val),
            _ => None,
        }
    }
}
