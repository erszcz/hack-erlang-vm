use beam;
use std::collections::HashMap;
use std::num::Int;
use std::path::Path;

pub type AtomIndex = usize;
pub type Atom = String;

pub struct AtomTable {
    // index to atom
    i_to_a: Vec<Atom>,
    // atom to index
    a_to_i: HashMap<Atom, AtomIndex>
}

impl AtomTable {

    pub fn from_chunk(chunk: &beam::Chunk) -> AtomTable {
        let ref data = chunk.data;
        let no_of_atoms = unsafe {
            // Range 0..4 since sizeof(u32) == 4.
            Int::from_be(*(data[0..4].as_slice().as_ptr() as *const u32))
        };
        let mut i = 0;
        let mut offset = 4;
        let mut atoms = AtomTable::new();
        while offset < data.len() {
            let len = Int::from_be(data[offset]) as usize;
            let (from, to) = (offset + 1, offset + 1 + len);
            let atom = String::from_utf8_lossy(&data[from..to]).into_owned();
            atoms.add(atom);
            offset += 1 + len;
        }
        atoms
    }

    fn new() -> AtomTable { AtomTable { i_to_a: vec![], a_to_i: HashMap::new() } }

    pub fn list(&self) -> Vec<(AtomIndex, Atom)> {
        self.i_to_a.iter().map(|i| i.clone()).enumerate().collect()
    }

    fn add(&mut self, atom: Atom) {
        if self.a_to_i.contains_key(&atom)
            { return }
        let index = self.i_to_a.len() as AtomIndex;
        self.i_to_a.push(atom.clone());
        self.a_to_i.insert(atom, index);
    }

    fn get(&self, atom: Atom) -> Option<AtomIndex> {
        match self.a_to_i.get(&atom) {
            Some (index) => Some (*index),
            None => None
        }
    }

}

#[test]
fn test_atom_table_from_chunk() {
    let expected_atoms: Vec<(usize, String)> =
        [(0, "fac"),
         (1, "state"),
         (2, "erlang"),
         (3, "-"),
         (4, "*"),
         (5, "module_info"),
         (6, "get_module_info")]
             .iter().map(|&(i,s)| (i,s.to_string())).collect();
    let path = Path::new("../erlang/fac.beam");
    if let Ok (beam) = beam::Beam::from_file(&path) {
        let atoms = AtomTable::from_chunk(beam.chunk("Atom").expect("can't get chunk"));
        assert_eq!(expected_atoms, atoms.list());
    } else {
        panic!("can't read .beam file")
    }
}

#[test]
fn add_atom() {
    let mut atoms = AtomTable::new();
    atoms.add("atom0".to_string());
    // should pass without panicking
}

#[test]
fn get_atom_index() {
    let mut atoms = AtomTable::new();
    atoms.add("atom0".to_string());
    assert_eq!(Some (0), atoms.get("atom0".to_string()));
}

#[test]
fn list_atoms() {
    let mut atoms = AtomTable::new();
    atoms.add("atom0".to_string());
    atoms.add("atom1".to_string());
    assert_eq!(vec![(0, "atom0".to_string()),
                    (1, "atom1".to_string())],
               atoms.list());
}
