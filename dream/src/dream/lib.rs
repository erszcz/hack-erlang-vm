#![feature(collections)]
#![feature(core)]
#![feature(fs)]
#![feature(io)]
#![feature(path)]

use std::collections::HashMap;

mod atoms;
mod beam;

struct Emu {
    pub atoms:      atoms::AtomTable,
    pub exports:    ExportTable,
    //pub code:       CodeTable
}

type Module = atoms::AtomIndex;
type Function = atoms::AtomIndex;
type Arity = usize;
type MFA = (Module, Function, Arity);
type CodeIndex = usize;

struct ExportTable {
    mfa_to_ci: HashMap<MFA, CodeIndex>
}

impl ExportTable {

    fn new() -> ExportTable {
        ExportTable { mfa_to_ci: HashMap::new() }
    }

    fn put(&mut self, mfa: MFA, code_index: CodeIndex) {
        self.mfa_to_ci.insert(mfa, code_index);
    }

    fn get(&self, mfa: MFA) -> Option<CodeIndex> {
        match self.mfa_to_ci.get(&mfa) {
            Some (index) => Some (*index),
            None => None
        }
    }

    fn list(&self) -> Vec<(MFA, CodeIndex)> {
        self.mfa_to_ci.iter().map(|(k,v)| (*k, *v)).collect()
    }

}

#[test]
fn put_exported_function() {
    let mut et = ExportTable::new();
    et.put((0, 0, 0), 0);
    // should pass without panicking
}

#[test]
fn get_exported_function() {
    let mut et = ExportTable::new();
    let mfa = (0, 0, 0);
    et.put(mfa, 0);
    assert_eq!(Some (0), et.get(mfa));
}

#[test]
fn list_exports() {
    let mut et = ExportTable::new();
    let mfa1 = (0,0,0);
    let mfa2 = (0,1,2);
    et.put(mfa1, 0);
    et.put(mfa2, 1);
    let mut example = vec![(mfa1, 0), (mfa2, 1)];
    example.sort();
    let mut actual = et.list();
    actual.sort();
    assert_eq!(example, actual);
}
