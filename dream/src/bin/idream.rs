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
        "code-chunk" => print_code(args),
        "code-labels" => print_labels(args),
        "code-replaced" => print_replaced(args),
        _ => panic!(format!("unrecognized module subcommand: {:?}", subcommand))
    }
}

fn dispatch_rts(_subcommand: &str, _args: &[String]) {
    panic!("not implemented yet");
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

fn module_name(path: &Path) -> Option<&str> {
    path.file_stem().and_then(|module| module.to_str())
}

fn list_module_exports(args: &[String]) {
    let arg0 = args[0].to_string();
    let path = Path::new(&arg0);
    let module = module_name(&path).unwrap();
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

fn print_code(args: &[String]) {
    let arg0 = args[0].to_string();
    let path = Path::new(&arg0);
    let beam = Beam::from_file(path).unwrap();
    let raw_code_chunk = beam.chunk("Code").expect("no Code chunk");
    let code_chunk = dream::code::CodeChunk::from_chunk(&raw_code_chunk).unwrap();
    println!("{}{}",
             format_code_metadata(&code_chunk),
             format_code(&code_chunk.code));
}

fn print_labels(args: &[String]) {
    let arg0 = args[0].to_string();
    let path = Path::new(&arg0);
    let mut loader = dream::loader::State::new(path).unwrap();
    dream::loader::load_code(&mut loader);
    dream::loader::load_labels(&mut loader);
    let labels = loader.labels.unwrap();
    println!("{}", format_labels(&labels));
}

fn print_replaced(args: &[String]) {
    let arg0 = args[0].to_string();
    let path = Path::new(&arg0);
    let mut loader = dream::loader::State::new(path).unwrap();
    dream::loader::load_code(&mut loader);
    dream::loader::load_labels(&mut loader);
    dream::loader::replace_jumps(&mut loader);
    let code = loader.code.unwrap();
    println!("{}", format_code(&code));
}

fn format_code_metadata(code_chunk: &dream::code::CodeChunk) -> String {
    let mut s = String::new();
    s.push_str(&format!("id              : {}\n", code_chunk.id));
    s.push_str(&format!("len             : {}\n", code_chunk.len));
    s.push_str(&format!("info_fields_len : {}\n", code_chunk.info_fields_len));
    s.push_str(&format!("instruction_set : {}\n", code_chunk.instruction_set));
    s.push_str(&format!("opcode_max      : {}\n", code_chunk.opcode_max));
    s.push_str(&format!("n_labels        : {}\n", code_chunk.n_labels));
    s.push_str(&format!("n_functions     : {}\n", code_chunk.n_functions));
    s.push_str(&format!("code            :\n"));
    s
}

fn format_code(code: &Vec<dream::code::Op>) -> String {
    let mut s = String::new();
    for (i, op) in code.iter().enumerate() {
        let sep = if i == 0 { "" } else { "\n" };
        s.push_str(&format!("{}  {:5} {} {:?}",
                            sep, i, op.name(), op.args))
    }
    s
}

fn format_labels(labels: &Vec<(dream::Label, dream::CodeIdx)>) -> String {
    let mut s = String::new();
    for (i, &(label, idx)) in labels.iter().enumerate() {
        let sep = if i == 0 { "" } else { "\n" };
        s.push_str(&format!("{}label {:4} -> code index {:5}", sep, label, idx));
    }
    s
}
