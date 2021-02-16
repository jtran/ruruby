use crate::*;
///
///  Inline constant cache
///
///  This module supports inline constant cache which is embedded in the instruction sequence directly.
///
#[derive(Debug, Clone)]
pub struct ConstCache {
    table: Vec<ConstCacheEntry>,
    id: u32,
    #[cfg(feature = "perf-method")]
    pub total: usize,
    #[cfg(feature = "perf-method")]
    pub missed: usize,
}

#[derive(Debug, Clone)]
pub struct ConstCacheEntry {
    pub version: u32,
    pub val: Option<Value>,
}

impl ConstCacheEntry {
    pub fn new() -> Self {
        ConstCacheEntry {
            version: 0,
            val: None,
        }
    }
}

impl ConstCache {
    pub fn new() -> Self {
        ConstCache {
            table: vec![],
            id: 0,
            #[cfg(feature = "perf-method")]
            total: 0,
            #[cfg(feature = "perf-method")]
            missed: 0,
        }
    }
    pub fn add_entry(&mut self) -> u32 {
        self.id += 1;
        self.table.push(ConstCacheEntry::new());
        self.id - 1
    }

    pub fn get_entry(&self, id: u32) -> &ConstCacheEntry {
        &self.table[id as usize]
    }

    pub fn get_mut_entry(&mut self, id: u32) -> &mut ConstCacheEntry {
        &mut self.table[id as usize]
    }

    pub fn set(&mut self, id: u32, version: u32, val: Value) {
        self.table[id as usize] = ConstCacheEntry {
            version,
            val: Some(val),
        };
    }
}

#[cfg(feature = "perf-method")]
impl ConstCache {
    pub fn clear(&mut self) {
        self.missed = 0;
        self.total = 0;
    }

    pub fn print_stats(&self) {
        eprintln!("+-------------------------------------------+");
        eprintln!("| Constant cache stats:                     |");
        eprintln!("+-------------------------------------------+");
        eprintln!("  hit              : {:>10}", self.total - self.missed);
        eprintln!("  missed           : {:>10}", self.missed);
    }
}
