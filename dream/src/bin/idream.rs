#![cfg(not(test))]

extern crate docopt;
extern crate dream;
extern crate rustc_serialize;

use docopt::Docopt;
use dream::beam::Beam;
use std::path::Path;

static USAGE: &'static str = "
Interactive dream

Usage:
    idream <command> <args>...
    idream [options]

Options:
    -h, --help      Display this message
    --list          List installed commands
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_command: Option<Command>,
    arg_args: Vec<String>,
    flag_help: bool,
    flag_list: bool
}

#[derive(Debug, RustcDecodable)]
enum Command {
    Module,
    RTS
}

#[derive(Debug, RustcDecodable)]
enum Subcommand {
    Atoms,
    Exports
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                      .and_then(|d| d.decode())
                      .unwrap_or_else(|e| e.exit());
    // Either some command is specified...
    if let Some(command) = args.arg_command {
        match command {
            Command::Module =>
                dispatch_module(&args.arg_args[0], &args.arg_args[1..]),
            Command::RTS =>
                dispatch_rts(&args.arg_args[0], &args.arg_args[1..])
        }
    }
    // ..or there are options to handle.
    else {
        // Which we don't do yet.
        usage()
    }
}

fn usage() {
    // Skip the first character, i.e. a newline.
    print!("{}", &USAGE[1..]);
}

fn dispatch_module(subcommand: &str, args: &[String]) {
    match subcommand {
        "atoms" => list_atoms(args),
        "exports" => list_exports(args),
        _ => panic!(format!("unrecognized module subcommand: {:?}", subcommand))
    }
}

fn dispatch_rts(subcommand: &str, args: &[String]) {
}

fn list_atoms(args: &[String]) {
    let arg0 = args[0].to_string();
    let path = Path::new(&arg0);
    let beam = Beam::from_file(path).unwrap();
    let atoms = dream::atoms::AtomTable::from_chunk(beam.chunk("Atom").expect("no Atom chunk"));
    print_atoms(&atoms);
}

fn print_atoms(atoms: &dream::atoms::AtomTable) {
    for &(ref index, ref atom) in atoms.list().iter() {
        println!("{} {}", index, atom);
    }
}

fn list_exports(args: &[String]) {
    let arg0 = args[0].to_string();
    let path = Path::new(&arg0);
    let mut emu = dream::Emu::new();
    if let Ok (()) = emu.load_module(path) {
        for &((m,f,a), ref label) in emu.exports.list().iter() {
            let module = emu.atoms.get_atom(m).expect("module name not in atom table");
            let function = emu.atoms.get_atom(f).expect("function name not in atom table");
            println!("{}:{}/{} ({},{},{}) at {}", module, function, a, m, f, a, label);
        }
    } else {
        panic!("can't load module")
    }
}
