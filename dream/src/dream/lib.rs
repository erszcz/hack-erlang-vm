pub mod atoms;
pub mod beam;
pub mod code;
pub mod exports;
pub mod loader;

pub use atoms::AtomTable;
pub use beam::{ Beam, Chunk };
pub use exports::{ CodeIdx, ExportTable };

pub type Label = u32;

use std::path::Path;

pub struct Emu {
    pub atoms:      AtomTable,
    pub exports:    ExportTable,
    //pub code:       CodeTable
}

impl Emu {

    pub fn new() -> Emu {
        Emu { atoms: AtomTable::new(),
              exports: ExportTable::new() }
    }

    pub fn load_module(&mut self, path: &Path) -> Result<(), String> {
        let modname = try!( modname_from_path(path) );
        let beam = try!( Beam::from_file(path) );
        let atom_chunk = try!( get_chunk(&beam, "Atom") );
        let expt_chunk = try!( get_chunk(&beam, "ExpT") );
        let mod_atoms = AtomTable::from_chunk(atom_chunk);
        load_atoms(self, &mod_atoms);
        load_exports(self, &modname, &mod_atoms, expt_chunk);
        Ok (())
    }

}

fn get_chunk<'beam>(beam: &'beam Beam,
                    chunk: &str) -> Result<&'beam Chunk, String> {
    beam.chunk(chunk).ok_or(format!("chunk {:?} not found", chunk))
}

fn load_atoms(emu: &mut Emu, mod_atoms: &AtomTable) {
    for &(_, ref atom) in mod_atoms.list().iter() {
        emu.atoms.add(atom);
    }
}

fn load_exports(emu: &mut Emu, modname: &String,
                mod_atoms: &AtomTable, expt_chunk: &Chunk) {
    let ref mut atoms = emu.atoms;
    let ref mut exports = emu.exports;
    let modname_emu_index = lookup_or_add(atoms, modname);
    let mod_exports = exports::from_chunk(expt_chunk);
    for mod_export in mod_exports {
        let fun = mod_atoms.get_atom(mod_export.function as atoms::AtomIndex)
            .expect("atom not found");
        let fun_emu_index = lookup_or_add(atoms, &fun);
        let mfa = (modname_emu_index as atoms::AtomIndex,
                   fun_emu_index as atoms::AtomIndex,
                   mod_export.arity as exports::Arity);
        exports.put(mfa, mod_export.label);
    }
}

fn modname_from_path(path: &Path) -> Result<String, String> {
    path.file_stem()
        .and_then(|modname| Some( modname.to_string_lossy().into_owned() ))
        .ok_or(format!("can't build module name from {:?}", path))
}

fn lookup_or_add(atoms: &mut AtomTable, atom: &String) -> atoms::AtomIndex {
    atoms.add(atom)
}

#[test]
fn test_modname_from_path() {
    let path = Path::new("path/to/enlightenment.beam");
    match modname_from_path(path) {
        Err (e) => panic!(e),
        Ok (modname) => {
            assert_eq!("enlightenment".to_string(), modname)
        }
    }
}
