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
        }
    }

    #[cfg(feature = "perf-method")]
    pub fn print_stats() {
        let (all, miss) = IVAR_CACHE.with(|m| (m.borrow().inline_all, m.borrow().inline_miss));
        eprintln!("+-------------------------------------------+");
        eprintln!("| Ivar cache stats:                         |");
        eprintln!("+-------------------------------------------+");
        eprintln!("  hit              : {:>10}", all - miss);
        eprintln!("  missed           : {:>10}", miss);
    }

    pub fn new_accessor() -> AccesorSlot {
        IVAR_CACHE.with(|m| {
            let cache = &mut m.borrow_mut().accessor;
            cache.push(None);
            AccesorSlot::new((cache.len() - 1) as u32)
        })
    }

    pub fn get_accessor(receiver_class: Module, name: IdentId, slot: AccesorSlot) -> IvarSlot {
        IVAR_CACHE.with(|m| {
            #[cfg(feature = "perf-method")]
            {
                m.borrow_mut().inline_all += 1;
            }
            let slot = {
                let entry = &mut m.borrow_mut().accessor[slot.into_usize()];
                if let Some(iv_slot) = entry {
                    return *iv_slot;
                };
                let iv_slot = receiver_class.get_ivar_slot(name);
                *entry = Some(iv_slot);
                iv_slot
            };
            #[cfg(feature = "perf-method")]
            {
                m.borrow_mut().inline_miss += 1;
            }
            slot
        })
    }

    pub fn new_inline() -> IvarInlineSlot {
        IVAR_CACHE.with(|m| {
            let cache = &mut m.borrow_mut().inline;
            cache.push(InlineIVCacheEntry {
                class: Module::default(),
                iv_slot: IvarSlot::new(0),
            });
            IvarInlineSlot::new((cache.len() - 1) as u32)
        })
    }

    pub fn get_inline(class: Module, name: IdentId, slot: IvarInlineSlot) -> IvarSlot {
        IVAR_CACHE.with(|m| {
            #[cfg(feature = "perf-method")]
            {
                m.borrow_mut().inline_all += 1;
            }
            let slot = {
                let entry = &mut m.borrow_mut().inline[slot.into_usize()];
                if entry.class.id() == class.id() {
                    return entry.iv_slot;
                };
                let iv_slot = class.get_ivar_slot(name);
                *entry = InlineIVCacheEntry { class, iv_slot };
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
    class: Module,
    iv_slot: IvarSlot,
}
