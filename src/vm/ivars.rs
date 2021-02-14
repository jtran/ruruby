use crate::*;
use std::cell::RefCell;

thread_local!(
    static IVAR_CACHE: RefCell<IvarCache> = RefCell::new(IvarCache::new());
);

#[derive(Debug)]
pub struct IvarCache {
    inline: Vec<InlineIVCacheEntry>,
}

impl IvarCache {
    fn new() -> Self {
        Self { inline: vec![] }
    }

    pub fn new_inline() -> u32 {
        IVAR_CACHE.with(|m| {
            let mut cache = m.borrow_mut();
            cache.inline.push(InlineIVCacheEntry {
                class: Module::default(),
                iv_slot: IvarSlot::new(0),
            });
            (cache.inline.len() - 1) as u32
        })
    }

    pub fn get_inline(class: Module, slot: u32) -> Option<IvarSlot> {
        IVAR_CACHE.with(|m| {
            let entry = &mut m.borrow_mut().inline[slot as usize];
            if entry.class.id() == class.id() {
                Some(entry.iv_slot)
            } else {
                None
            }
        })
    }

    pub fn update_inline(class: Module, iv_slot: IvarSlot, slot: u32) {
        IVAR_CACHE.with(|m| {
            m.borrow_mut().inline[slot as usize] = InlineIVCacheEntry { class, iv_slot };
        });
    }
}

#[derive(Debug)]
struct InlineIVCacheEntry {
    class: Module,
    iv_slot: IvarSlot,
}
