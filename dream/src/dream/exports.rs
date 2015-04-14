use std;
use std::collections::HashMap;
use super::atoms;
use super::beam;

pub type Module = atoms::AtomIndex;
pub type Function = atoms::AtomIndex;
pub type Arity = usize;
pub type MFA = (Module, Function, Arity);
pub type CodeIndex = usize;

pub struct ExportTable {
    mfa_to_ci: HashMap<MFA, CodeIndex>
}

impl ExportTable {

    pub fn new() -> ExportTable {
        ExportTable { mfa_to_ci: HashMap::new() }
    }

    pub fn put(&mut self, mfa: MFA, code_index: CodeIndex) {
        self.mfa_to_ci.insert(mfa, code_index);
    }

    pub fn get(&self, mfa: MFA) -> Option<CodeIndex> {
        match self.mfa_to_ci.get(&mfa) {
            Some (index) => Some (*index),
            None => None
        }
    }

    pub fn list(&self) -> Vec<(MFA, CodeIndex)> {
        self.mfa_to_ci.iter().map(|(k,v)| (*k, *v)).collect()
    }

}

pub fn from_chunk(chunk: &beam::Chunk) -> Vec<ChunkExport> {
    let mut exports = vec![];
    for export_data in chunk.data[4..].chunks(12) {
        if export_data.len() < 12
            // Last incomplete chunk, probably padding not an exported function.
            { continue }
        let export = ChunkExport::from_slice(export_data);
        exports.push(export);
    }
    exports
}

pub struct ChunkExport {
    pub function: u32,
    pub arity: u32,
    pub label: u32
}

impl ChunkExport {

    fn from_slice(data: &[u8]) -> ChunkExport {
        let mut export: ChunkExport = unsafe { std::mem::uninitialized() };
        unsafe {
            std::ptr::copy(&data[0],
                           &mut export as *mut ChunkExport as *mut u8,
                           std::mem::size_of::<ChunkExport>());
        }
        export.function = u32::from_be(export.function);
        export.arity = u32::from_be(export.arity);
        export.label = u32::from_be(export.label);
        export
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
