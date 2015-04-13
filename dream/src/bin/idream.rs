extern crate dream;

use dream::beam::Beam;
use std::path::Path;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let ref command = args[1];
    match command.as_slice() {
        "atoms" => list_atoms(args[2..].as_slice()),
        _ => usage()
    }
}

fn list_atoms(args: &[String]) {
    let ref arg0 = args[0];
    let path = Path::new(arg0.as_slice());
    let beam = Beam::from_file(path).unwrap();
    let atoms = dream::atoms::AtomTable::from_chunk(beam.chunk("Atom").expect("no Atom chunk"));
    for &(ref index, ref atom) in atoms.list().iter() {
        println!("{} {}", index, atom);
    }
}

fn usage() {
    panic!("u r doin it wrong!")
}
