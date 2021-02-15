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
}

impl IvarCache {
    fn new() -> Self {
        Self {
            inline: vec![],
            #[cfg(feature = "perf-method")]
            inline_all: 0,
            #[cfg(feature = "perf-method")]
            inline_miss: 0,
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

    pub fn new_inline() -> u32 {
        IVAR_CACHE.with(|m| {
            let cache = &mut m.borrow_mut().inline;
            cache.push(InlineIVCacheEntry {
                class: Module::default(),
                iv_slot: IvarSlot::new(0),
            });
            (cache.len() - 1) as u32
        })
    }

    pub fn get_inline(class: Module, name: IdentId, slot: u32) -> IvarSlot {
        IVAR_CACHE.with(|m| {
            #[cfg(feature = "perf-method")]
            {
                m.borrow_mut().inline_all += 1;
            }
            let slot = {
                let entry = &mut m.borrow_mut().inline[slot as usize];
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
