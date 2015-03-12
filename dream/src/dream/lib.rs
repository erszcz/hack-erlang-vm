use std::collections::HashMap;

mod beam;

struct Emu {
    pub atoms: AtomTable,
    pub exports: ExportTable,
    //pub code: CodeTable
}

type AtomIndex = usize;
type Atom = String;

struct AtomTable {
    // index to atom
    i_to_a: Vec<Atom>,
    // atom to index
    a_to_i: HashMap<Atom, AtomIndex>
}

type Module = AtomIndex;
type Function = AtomIndex;
type Arity = usize;
type MFA = (Module, Function, Arity);
type CodeIndex = usize;

struct ExportTable {
    mfa_to_ci: HashMap<MFA, CodeIndex>
}

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
