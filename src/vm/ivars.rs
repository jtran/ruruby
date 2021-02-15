use crate::*;
use std::cell::RefCell;

thread_local!(
    static IVAR_CACHE: RefCell<IvarCache> = RefCell::new(IvarCache::new());
);

#[derive(Debug)]
pub struct IvarCache {
    inline: Vec<InlineIVCacheEntry>,
    #[cfg(feature = "perf-method")]
    inline_all: usize,
    #[cfg(feature = "perf-method")]
    inline_miss: usize,
    accessor: Vec<Option<IvarSlot>>,
    #[cfg(feature = "perf-method")]
    accessor_all: usize,
    #[cfg(feature = "perf-method")]
    accessor_miss: usize,
}
#[derive(Debug, Clone, Copy)]
pub struct AccesorSlot(u32);

impl AccesorSlot {
    pub fn new(slot: u32) -> Self {
        Self(slot)
    }

    pub fn into_usize(self) -> usize {
        self.0 as usize
    }
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
        Self {
            inline: vec![],
            #[cfg(feature = "perf-method")]
            inline_all: 0,
            #[cfg(feature = "perf-method")]
            inline_miss: 0,
            accessor: vec![],
            #[cfg(feature = "perf-method")]
            accessor_all: 0,
            #[cfg(feature = "perf-method")]
            accessor_miss: 0,
        }
    }

    #[cfg(feature = "perf-method")]
    pub fn print_stats() {
        let (inline_all, inline_miss, accessor_all, accessor_miss) = IVAR_CACHE.with(|m| {
            let c = m.borrow();
            (c.inline_all, c.inline_miss, c.accessor_all, c.accessor_miss)
        });
        eprintln!("+-------------------------------------------+");
        eprintln!("| Ivar cache stats:                         |");
        eprintln!("+-------------------------------------------+");
        eprintln!("  inline hit        : {:>10}", inline_all - inline_miss);
        eprintln!("  inline missed     : {:>10}", inline_miss);
        eprintln!("  accessor hit      : {:>10}", accessor_all - accessor_miss);
        eprintln!("  accessor missed   : {:>10}", accessor_miss);
    }

    pub fn new_accessor() -> AccesorSlot {
        IVAR_CACHE.with(|m| {
            let cache = &mut m.borrow_mut().accessor;
            cache.push(None);
            AccesorSlot::new((cache.len() - 1) as u32)
        })
    }

    pub fn get_accessor(ary: &mut IvarInfo, name: IdentId, slot: AccesorSlot) -> IvarSlot {
        IVAR_CACHE.with(|m| {
            #[cfg(feature = "perf-method")]
            {
                m.borrow_mut().accessor_all += 1;
            }
            let slot = {
                let entry = &mut m.borrow_mut().accessor[slot.into_usize()];
                if let Some(iv_slot) = *entry {
                    return iv_slot;
                };
                let iv_slot = ary.get_ivar_slot(name);
                *entry = Some(iv_slot);
                iv_slot
            };
            #[cfg(feature = "perf-method")]
            {
                m.borrow_mut().accessor_miss += 1;
            }
            slot
        })
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
            {
                m.borrow_mut().inline_all += 1;
            }
            let slot = {
                let entry = &mut m.borrow_mut().inline[slot.into_usize()];
                if entry.class == Some(ary.class()) {
                    return entry.iv_slot;
                };
                let iv_slot = ary.get_ivar_slot(name);
                *entry = InlineIVCacheEntry {
                    class: Some(ary.class()),
                    iv_slot,
                };
                iv_slot
            };
            #[cfg(feature = "perf-method")]
            {
                m.borrow_mut().inline_miss += 1;
            }
            slot
        })
    }
}

#[derive(Debug)]
struct InlineIVCacheEntry {
    class: Option<ClassRef>,
    iv_slot: IvarSlot,
}
