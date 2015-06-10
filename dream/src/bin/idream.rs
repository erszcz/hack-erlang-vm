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
        "atoms" => list_module_atoms(args),
        "exports" => list_module_exports(args),
        _ => panic!(format!("unrecognized module subcommand: {:?}", subcommand))
    }
}

fn dispatch_rts(subcommand: &str, args: &[String]) {
}

fn list_module_atoms(args: &[String]) {
    let arg0 = args[0].to_string();
    let path = Path::new(&arg0);
    let beam = Beam::from_file(path).unwrap();
    let atoms = dream::atoms::AtomTable::from_chunk(beam.chunk("Atom").expect("no Atom chunk"));
    print!("{}", format_atoms(&atoms));
}

fn format_atoms(atoms: &dream::atoms::AtomTable) -> String {
    let mut s = String::new();
    for &(ref index, ref atom) in atoms.list().iter() {
        s.push_str(&format!("{} {}\n", index, atom));
    }
    s
}

fn list_module_exports(args: &[String]) {
    let arg0 = args[0].to_string();
    let path = Path::new(&arg0);
    let module = path.file_stem().and_then(|module| module.to_str()).unwrap();
    let beam = Beam::from_file(path).unwrap();
    let atoms = dream::atoms::AtomTable::from_chunk(beam.chunk("Atom")
                                                        .expect("no Atom chunk"));
    let expt_chunk = beam.chunk("ExpT").expect("no ExpT chunk");
    for export in dream::exports::from_chunk(expt_chunk) {
        let function = atoms.get_atom(export.function as usize)
                            .expect("function name not found in atom table");
        let module_name_index = atoms.get_index(module)
                                     .expect("module name not found in atom table");
        println!("{}:{}/{} ({},{},{}) at {}",
                 module, function, export.arity,
                 module_name_index, export.function, export.arity, export.label);
    }
}
