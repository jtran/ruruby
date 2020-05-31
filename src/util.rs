use core::ptr::NonNull;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Annot<T> {
    pub kind: T,
    pub loc: Loc,
}

impl<T> Annot<T> {
    pub fn new(kind: T, loc: Loc) -> Self {
        Annot { kind, loc }
    }

    pub fn loc(&self) -> Loc {
        self.loc
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loc(pub u32, pub u32);

impl Loc {
    pub fn new(loc: Loc) -> Self {
        loc
    }

    pub fn dec(&self) -> Self {
        use std::cmp::*;
        Loc(min(self.0, self.1 - 1), self.1 - 1)
    }

    pub fn merge(&self, loc: Loc) -> Self {
        use std::cmp::*;
        Loc(min(self.0, loc.0), max(self.1, loc.1))
    }
}

//------------------------------------------------------------

#[derive(Debug)]
pub struct Ref<T>(NonNull<T>);

impl<T> Ref<T> {
    pub fn new(info: T) -> Self {
        let boxed = Box::into_raw(Box::new(info));
        Ref(NonNull::new(boxed).unwrap_or_else(|| panic!("Ref::new(): the pointer is NULL.")))
    }

    pub fn from_ref(info: &T) -> Self {
        Ref(NonNull::new(info as *const T as *mut T)
            .unwrap_or_else(|| panic!("Ref::from_ref(): the pointer is NULL.")))
    }

    pub fn from_ptr(info: *mut T) -> Self {
        Ref(NonNull::new(info).unwrap_or_else(|| panic!("Ref::from_ptr(): the pointer is NULL.")))
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    pub fn inner(&self) -> &T {
        unsafe { &*self.0.as_ptr() }
    }

    pub fn inner_mut(&self) -> &mut T {
        unsafe { &mut *self.0.as_ptr() }
    }

    pub fn id(&self) -> u64 {
        self.0.as_ptr() as u64
    }
}

impl<T: Clone> Ref<T> {
    /// Allocates a copy of `self<T>` on the heap, returning `Ref`.
    pub fn dup(&self) -> Self {
        Self::new(self.inner().clone())
    }
}

unsafe impl<T> Send for Ref<T> {}

impl<T> Copy for Ref<T> {}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Ref<T> {}

impl<T> std::hash::Hash for Ref<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> std::ops::Deref for Ref<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.as_ptr() }
    }
}

impl<T> std::ops::DerefMut for Ref<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.as_ptr() }
    }
}

//------------------------------------------------------------

pub type SourceInfoRef = Ref<SourceInfo>;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceInfo {
    pub path: PathBuf,
    pub code: Vec<char>,
}

impl SourceInfoRef {
    pub fn empty() -> Self {
        SourceInfoRef::new(SourceInfo::new(PathBuf::default()))
    }
}

impl SourceInfo {
    pub fn new(path: PathBuf) -> Self {
        SourceInfo {
            path: path,
            code: vec![],
        }
    }
    pub fn show_file_name(&self) {
        eprintln!("{}", self.path.to_string_lossy());
    }

    /// Show the location of the Loc in the source code using '^^^'.
    pub fn show_loc(&self, loc: &Loc) {
        let mut line: u32 = 1;
        let mut line_top_pos: u32 = 0;
        let mut line_pos = vec![];
        for (pos, ch) in self.code.iter().enumerate() {
            if *ch == '\n' {
                line_pos.push((line, line_top_pos, pos as u32));
                line += 1;
                line_top_pos = pos as u32 + 1;
            }
        }
        if line_top_pos as usize <= self.code.len() - 1 {
            line_pos.push((line, line_top_pos, self.code.len() as u32 - 1));
        }

        let mut found = false;
        for line in &line_pos {
            if line.2 < loc.0 || line.1 > loc.1 {
                continue;
            }
            if !found {
                eprintln!("{}:{}", self.path.to_string_lossy(), line.0);
            };
            found = true;
            let start = line.1 as usize;
            let mut end = line.2 as usize;
            if self.code[end] == '\n' {
                end -= 1
            }
            eprintln!("{}", self.code[start..=end].iter().collect::<String>());
            use std::cmp::*;
            let read = if loc.0 <= line.1 {
                0
            } else {
                self.code[(line.1 as usize)..(loc.0 as usize)]
                    .iter()
                    .map(|x| calc_width(x))
                    .sum()
            };
            let length: usize = self.code[max(loc.0, line.1) as usize..min(loc.1, line.2) as usize]
                .iter()
                .map(|x| calc_width(x))
                .sum();
            eprintln!("{}{}", " ".repeat(read), "^".repeat(length + 1));
        }

        if !found {
            let line = match line_pos.last() {
                Some(line) => (line.0 + 1, line.2 + 1, loc.1),
                None => (1, 0, loc.1),
            };
            let read = self.code[(line.1 as usize)..(loc.0 as usize)]
                .iter()
                .map(|x| calc_width(x))
                .sum();
            let length: usize = self.code[loc.0 as usize..loc.1 as usize]
                .iter()
                .map(|x| calc_width(x))
                .sum();
            let is_cr = loc.1 as usize >= self.code.len() || self.code[loc.1 as usize] == '\n';
            eprintln!("{}:{}", self.path.to_string_lossy(), line.0);
            eprintln!(
                "{}",
                if !is_cr {
                    self.code[(line.1 as usize)..=(loc.1 as usize)]
                        .iter()
                        .collect::<String>()
                } else {
                    self.code[(line.1 as usize)..(loc.1 as usize)]
                        .iter()
                        .collect::<String>()
                }
            );
            eprintln!("{}{}", " ".repeat(read), "^".repeat(length + 1));
        }

        fn calc_width(ch: &char) -> usize {
            if ch.is_ascii() {
                1
            } else {
                2
            }
        }
    }
}

//------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentId(std::num::NonZeroU32);

impl std::hash::Hash for IdentId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Into<usize> for IdentId {
    fn into(self) -> usize {
        self.0.get() as usize
    }
}

impl Into<u32> for IdentId {
    fn into(self) -> u32 {
        self.0.get()
    }
}

impl From<u32> for IdentId {
    fn from(id: u32) -> Self {
        let id = unsafe { std::num::NonZeroU32::new_unchecked(id) };
        IdentId(id)
    }
}

pub struct OptionalId(Option<IdentId>);

impl OptionalId {
    pub fn new(id: impl Into<Option<IdentId>>) -> Self {
        OptionalId(id.into())
    }
}

impl std::ops::Deref for OptionalId {
    type Target = Option<IdentId>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! id {
    ($constant:expr) => {
        IdentId(unsafe { std::num::NonZeroU32::new_unchecked($constant) })
    };
}

impl IdentId {
    pub const INITIALIZE: IdentId = id!(1);
    pub const OBJECT: IdentId = id!(2);
    pub const NEW: IdentId = id!(3);
    pub const NAME: IdentId = id!(4);
    pub const _ADD: IdentId = id!(5);
    pub const _SUB: IdentId = id!(6);
    pub const _MUL: IdentId = id!(7);
    pub const _POW: IdentId = id!(8);
    pub const _SHL: IdentId = id!(9);
    pub const _REM: IdentId = id!(10);
    pub const _EQ: IdentId = id!(11);
    pub const _NEQ: IdentId = id!(12);
    pub const _GT: IdentId = id!(13);
    pub const _GE: IdentId = id!(14);
    pub const _DIV: IdentId = id!(15);
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierTable {
    table: HashMap<String, u32>,
    table_rev: HashMap<u32, String>,
    ident_id: u32,
}

impl IdentifierTable {
    pub fn new() -> Self {
        let mut table = IdentifierTable {
            table: HashMap::new(),
            table_rev: HashMap::new(),
            ident_id: 20,
        };
        table.set_ident_id("<null>", IdentId::from(0));
        table.set_ident_id("initialize", IdentId::INITIALIZE);
        table.set_ident_id("Object", IdentId::OBJECT);
        table.set_ident_id("new", IdentId::NEW);
        table.set_ident_id("name", IdentId::NAME);
        table.set_ident_id("+", IdentId::_ADD);
        table.set_ident_id("-", IdentId::_SUB);
        table.set_ident_id("*", IdentId::_MUL);
        table.set_ident_id("**", IdentId::_POW);
        table.set_ident_id("<<", IdentId::_SHL);
        table.set_ident_id("%", IdentId::_REM);
        table.set_ident_id("==", IdentId::_EQ);
        table.set_ident_id("!=", IdentId::_NEQ);
        table.set_ident_id(">", IdentId::_GT);
        table.set_ident_id(">=", IdentId::_GE);
        table.set_ident_id("/", IdentId::_DIV);
        table
    }

    fn set_ident_id(&mut self, name: impl Into<String>, id: IdentId) {
        let name = name.into();
        self.table.insert(name.clone(), id.into());
        self.table_rev.insert(id.into(), name);
    }

    pub fn get_ident_id(&mut self, name: impl Into<String>) -> IdentId {
        let name = name.into();
        match self.table.get(&name) {
            Some(id) => IdentId::from(*id),
            None => {
                let id = self.ident_id;
                self.table.insert(name.clone(), id);
                self.table_rev.insert(id, name.clone());
                self.ident_id += 1;
                IdentId::from(id)
            }
        }
    }

    pub fn get_name(&self, id: IdentId) -> &str {
        self.table_rev.get(&id.0.get()).unwrap()
    }

    pub fn add_postfix(&mut self, id: IdentId, postfix: &str) -> IdentId {
        let new_name = self.get_name(id).to_string() + postfix;
        self.get_ident_id(new_name)
    }
}
