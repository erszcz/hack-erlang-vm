use beam;
use std::collections::HashMap;

// TODO: remove `allow` clause once the bug with unused, though used, Path is solved
#[allow(unused_imports)]
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
        let mut offset = 4;
        let mut atoms = AtomTable::new();
        while offset < data.len() {
            let len = u8::from_be(data[offset]) as usize;
            let (from, to) = (offset + 1, offset + 1 + len);
            let atom = String::from_utf8_lossy(&data[from..to]).into_owned();
            atoms.add(&atom);
            offset += 1 + len;
        }
        atoms
    }

    pub fn new() -> AtomTable {
        let mut i_to_a = vec![];
        i_to_a.push("".to_string());
        AtomTable { i_to_a: i_to_a,
                    a_to_i: HashMap::new() }
    }

    pub fn list(&self) -> Vec<(AtomIndex, Atom)> {
        self.i_to_a.iter().map(|i| i.clone()).enumerate().skip(1).collect()
    }

    pub fn add(&mut self, atom: &str) -> AtomIndex {
        if let Some (index) = self.get_index(atom)
            { index }
        else {
            let index = self.i_to_a.len() as AtomIndex;
            self.i_to_a.push(atom.to_string());
            self.a_to_i.insert(atom.to_string(), index);
            index
        }
    }

    pub fn get_atom(&self, index: AtomIndex) -> Option<Atom> {
        if index < self.i_to_a.len()
            { Some (self.i_to_a[index].clone()) }
        else
            { None }
    }

    pub fn get_index(&self, atom: &str) -> Option<AtomIndex> {
        match self.a_to_i.get(atom) {
            Some (index) => Some (*index),
            None => None
        }
    }

}

#[test]
fn test_atom_table_from_chunk() {
    let expected_atoms: Vec<(usize, String)> =
        [(1, "fac"),
         (2, "state"),
         (3, "erlang"),
         (4, "-"),
         (5, "*"),
         (6, "module_info"),
         (7, "get_module_info")]
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
    assert_eq!(1, atoms.add("atom1"));
}

#[test]
fn get_atom_index() {
    let mut atoms = AtomTable::new();
    atoms.add("atom1");
    assert_eq!(Some (1), atoms.get_index("atom1"));
}

#[test]
fn list_atoms() {
    let mut atoms = AtomTable::new();
    atoms.add("atom1");
    atoms.add("atom2");
    assert_eq!(vec![(1, "atom1".to_string()),
                    (2, "atom2".to_string())],
               atoms.list());
}
