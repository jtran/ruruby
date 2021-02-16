use crate::*;

#[derive(Debug, Clone)]
pub struct IvarCache {
    inline: Vec<InlineIVCacheEntry>,
}

#[derive(Debug, Clone)]
struct InlineIVCacheEntry {
    class: Option<ClassRef>,
    iv_slot: IvarSlot,
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
    pub fn new() -> Self {
        Self { inline: vec![] }
    }

    pub fn get_accessor(
        mut ext: ClassRef,
        method: MethodId,
        name: IdentId,
        slot: Option<IvarSlot>,
    ) -> IvarSlot {
        #[cfg(feature = "perf-method")]
        Perf::inc_accessor_all();

        if let Some(iv_slot) = slot {
            return iv_slot;
        };
        let iv_slot = ext.get_ivar_slot(name);
        MethodRepo::update_accessor(method, iv_slot);

        #[cfg(feature = "perf-method")]
        Perf::inc_accessor_miss();
        iv_slot
    }

    pub fn new_inline(&mut self) -> IvarInlineSlot {
        let cache = &mut self.inline;
        cache.push(InlineIVCacheEntry {
            class: None,
            iv_slot: IvarSlot::new(0),
        });
        IvarInlineSlot::new((cache.len() - 1) as u32)
    }

    pub fn get_inline(
        &mut self,
        mut ext: ClassRef,
        name: IdentId,
        slot: IvarInlineSlot,
    ) -> IvarSlot {
        #[cfg(feature = "perf-method")]
        Perf::inc_inline_all();

        let entry = &mut self.inline[slot.into_usize()];
        if entry.class == Some(ext) {
            return entry.iv_slot;
        };
        let iv_slot = ext.get_ivar_slot(name);
        *entry = InlineIVCacheEntry {
            class: Some(ext),
            iv_slot,
        };

        #[cfg(feature = "perf-method")]
        Perf::inc_inline_miss();
        iv_slot
    }
}
