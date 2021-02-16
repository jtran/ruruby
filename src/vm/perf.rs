use super::vm_inst::Inst;
use std::time::{Duration, Instant};

use std::cell::RefCell;

thread_local!(
    pub static PERF: RefCell<Perf> = RefCell::new(Perf::new());
);

#[derive(Debug, Clone)]
pub struct PerfCounter {
    pub count: u64,
    pub duration: Duration,
}

impl PerfCounter {
    pub fn new() -> Self {
        PerfCounter {
            count: 0,
            duration: Duration::from_secs(0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Perf {
    counter: Vec<PerfCounter>,
    timer: Instant,
    prev_time: Duration,
    prev_inst: u8,
    #[cfg(feature = "perf-method")]
    inline_all: usize,
    #[cfg(feature = "perf-method")]
    inline_miss: usize,
    #[cfg(feature = "perf-method")]
    accessor_all: usize,
    #[cfg(feature = "perf-method")]
    accessor_miss: usize,
}

impl Perf {
    pub const GC: u8 = 252;
    pub const CODEGEN: u8 = 253;
    pub const EXTERN: u8 = 254;
    pub const INVALID: u8 = 255;
}

impl Perf {
    pub fn new() -> Self {
        Perf {
            counter: vec![PerfCounter::new(); 256],
            timer: Instant::now(),
            prev_time: Duration::from_secs(0),
            prev_inst: Perf::INVALID,
            #[cfg(feature = "perf-method")]
            inline_all: 0,
            #[cfg(feature = "perf-method")]
            inline_miss: 0,
            #[cfg(feature = "perf-method")]
            accessor_all: 0,
            #[cfg(feature = "perf-method")]
            accessor_miss: 0,
        }
    }

    /// Record duration for current instruction.
    pub fn get_perf(next_inst: u8) {
        PERF.with(|m| {
            let mut perf = m.borrow_mut();
            let prev = perf.prev_inst;
            assert!(next_inst != 0);
            assert!(prev != 0);
            let elapsed = perf.timer.elapsed();
            let prev_time = perf.prev_time;
            if prev != Perf::INVALID {
                perf.counter[prev as usize].count += 1;
                perf.counter[prev as usize].duration += elapsed - prev_time;
            }
            perf.prev_time = elapsed;
            perf.prev_inst = next_inst;
        })
    }

    pub fn get_perf_no_count(next_inst: u8) {
        Self::get_perf(next_inst);
        if next_inst != Perf::INVALID {
            PERF.with(|m| {
                m.borrow_mut().counter[next_inst as usize].count -= 1;
            })
        }
    }

    pub fn set_prev_inst(inst: u8) {
        PERF.with(|m| {
            m.borrow_mut().prev_inst = inst;
        })
    }

    pub fn get_prev_inst() -> u8 {
        PERF.with(|m| m.borrow().prev_inst)
    }

    pub fn print_perf() {
        eprintln!("+-------------------------------------------+");
        eprintln!("| Performance stats for inst:               |");
        eprintln!(
            "| {:<13} {:>9} {:>8} {:>8} |",
            "Inst name", "count", "%time", "ns/inst"
        );
        eprintln!("+-------------------------------------------+");
        PERF.with(|m| {
            let sum = m
                .borrow()
                .counter
                .iter()
                .fold(Duration::from_secs(0), |acc, x| acc + x.duration);
            for (
                i,
                PerfCounter {
                    count: c,
                    duration: d,
                },
            ) in m.borrow().counter.iter().enumerate()
            {
                if *c == 0 || i == 0 {
                    continue;
                }
                eprintln!(
                    "  {:<13}{:>9} {:>8.2} {:>8}",
                    if i as u8 == Perf::CODEGEN {
                        "CODEGEN".to_string()
                    } else if i as u8 == Perf::EXTERN {
                        "EXTERN".to_string()
                    } else if i as u8 == Perf::GC {
                        "GC".to_string()
                    } else {
                        Inst::inst_name(i as u8)
                    },
                    if *c > 10000_000 {
                        format!("{:>9}M", c / 1000_000)
                    } else if *c > 10000 {
                        format!("{:>9}K", c / 1000)
                    } else {
                        format!("{:>10}", *c)
                    },
                    (d.as_micros() as f64) * 100.0 / (sum.as_micros() as f64),
                    d.as_nanos() / (*c as u128)
                );
            }
        })
    }

    #[cfg(feature = "perf-method")]
    pub fn print_stats() {
        let (inline_all, inline_miss, accessor_all, accessor_miss) = PERF.with(|m| {
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

    #[cfg(feature = "perf-method")]
    pub fn inc_accessor_all() {
        PERF.with(|m| m.borrow_mut().accessor_all += 1);
    }

    #[cfg(feature = "perf-method")]
    pub fn inc_accessor_miss() {
        PERF.with(|m| m.borrow_mut().accessor_miss += 1);
    }

    #[cfg(feature = "perf-method")]
    pub fn inc_inline_all() {
        PERF.with(|m| m.borrow_mut().inline_all += 1);
    }

    #[cfg(feature = "perf-method")]
    pub fn inc_inline_miss() {
        PERF.with(|m| m.borrow_mut().inline_miss += 1);
    }
}
