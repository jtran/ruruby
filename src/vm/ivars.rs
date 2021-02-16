use crate::*;
use std::cell::RefCell;

thread_local!(
    static IVAR_CACHE: RefCell<IvarCache> = RefCell::new(IvarCache::new());
);

#[derive(Debug)]
pub struct IvarCache {
    inline: Vec<InlineIVCacheEntry>,
}

#[derive(Debug, Clone, Copy)]
pub struct IvarInlineSlot(u32);

impl IvarInlineSlot {
    pub fn new(slot: u32) -> Self {
        Self(slot)
    }

    pub fn into_usize(self) -> usize {
        self.0 as usize
    }
}

impl IvarCache {
    fn new() -> Self {
        Self { inline: vec![] }
    }

    pub fn get_accessor(
        ary: &mut IvarInfo,
        method: MethodId,
        name: IdentId,
        slot: Option<IvarSlot>,
    ) -> IvarSlot {
        #[cfg(feature = "perf-method")]
        Perf::inc_accessor_all();

        if let Some(iv_slot) = slot {
            return iv_slot;
        };
        let iv_slot = ary.get_ivar_slot(name);
        MethodRepo::update_accessor(method, iv_slot);

        #[cfg(feature = "perf-method")]
        Perf::inc_accessor_miss();
        iv_slot
    }

    pub fn new_inline() -> IvarInlineSlot {
        IVAR_CACHE.with(|m| {
            let cache = &mut m.borrow_mut().inline;
            cache.push(InlineIVCacheEntry {
                class: None,
                iv_slot: IvarSlot::new(0),
            });
            IvarInlineSlot::new((cache.len() - 1) as u32)
        })
    }

    pub fn get_inline(ary: &mut IvarInfo, name: IdentId, slot: IvarInlineSlot) -> IvarSlot {
        IVAR_CACHE.with(|m| {
            #[cfg(feature = "perf-method")]
            Perf::inc_inline_all();

            let entry = &mut m.borrow_mut().inline[slot.into_usize()];
            if entry.class == Some(ary.class()) {
                return entry.iv_slot;
            };
            let iv_slot = ary.get_ivar_slot(name);
            *entry = InlineIVCacheEntry {
                class: Some(ary.class()),
                iv_slot,
            };

            #[cfg(feature = "perf-method")]
            Perf::inc_inline_miss();
            iv_slot
        })
    }
}

#[derive(Debug)]
struct InlineIVCacheEntry {
    class: Option<ClassRef>,
    iv_slot: IvarSlot,
}
