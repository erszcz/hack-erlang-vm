#![cfg(not(test))]

extern crate dream;

use dream::beam::Beam;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ref command = args[1];
    match command.as_ref() {
        "atoms" => list_atoms(args[2..].as_ref()),
        _ => usage()
    }
}

fn list_atoms(args: &[String]) {
    let arg0 = args[0].to_string();
    let path = Path::new(&arg0);
    let beam = Beam::from_file(path).unwrap();
    let atoms = dream::atoms::AtomTable::from_chunk(beam.chunk("Atom").expect("no Atom chunk"));
    for &(ref index, ref atom) in atoms.list().iter() {
        println!("{} {}", index, atom);
    }
}

fn usage() {
    panic!("u r doin it wrong!")
}
