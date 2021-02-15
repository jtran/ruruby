use crate::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObjArray(Vec<Option<Value>>);

impl std::ops::Deref for ObjArray {
    type Target = Vec<Option<Value>>;
    fn deref(&self) -> &Vec<Option<Value>> {
        &self.0
    }
}

impl ObjArray {
    pub fn access(&mut self, slot: IvarSlot) -> Value {
        let slot = slot.into_usize();
        if self.len() <= slot {
            self.0.resize(slot + 1, None);
        }
        match self.0[slot] {
            Some(val) => val,
            None => Value::nil(),
        }
    }

    pub fn access_mut(&mut self, slot: IvarSlot) -> &mut Option<Value> {
        let slot = slot.into_usize();
        if self.len() <= slot {
            self.0.resize(slot + 1, None);
        }
        self.0[slot] = Some(Value::nil());
        unsafe { self.0.get_unchecked_mut(slot) }
    }
}

impl ObjArray {
    pub fn get(&self, slot: IvarSlot) -> Option<Value> {
        match self.0.get(slot.into_usize()) {
            Some(Some(val)) => Some(*val),
            _ => None,
        }
    }
}
