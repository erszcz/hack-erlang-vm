use std::collections::HashMap;

struct Emu {
    pub atoms: AtomTable,
    //pub exports: ExportTable,
    //pub code: CodeTable
}

struct AtomTable {
    // index to atom
    i_to_a: Vec<Atom>,
    // atom to index
    a_to_i: HashMap<Atom, AtomIndex>
}

type AtomIndex = usize;
type Atom = String;

impl AtomTable {

    fn new() -> AtomTable { AtomTable { i_to_a: vec![], a_to_i: HashMap::new() } }

    fn list(&self) -> Vec<(AtomIndex, Atom)> {
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
