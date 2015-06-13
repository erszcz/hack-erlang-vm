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
        "code" => print_code(args),
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
    println!("{}", format_code(&code_chunk));
}

fn format_code(code_chunk: &dream::code::CodeChunk) -> String {
    let mut s = String::new();
    s.push_str(&format!("id              : {}\n", code_chunk.id));
    s.push_str(&format!("len             : {}\n", code_chunk.len));
    s.push_str(&format!("info_fields_len : {}\n", code_chunk.info_fields_len));
    s.push_str(&format!("instruction_set : {}\n", code_chunk.instruction_set));
    s.push_str(&format!("opcode_max      : {}\n", code_chunk.opcode_max));
    s.push_str(&format!("n_labels        : {}\n", code_chunk.n_labels));
    s.push_str(&format!("n_functions     : {}\n", code_chunk.n_functions));
    s.push_str(&format!("code            :\n"));
    for (i, op) in operations(code_chunk).enumerate() {
        let sep = if i == 0 { "" } else { "\n" };
        s.push_str(&format!("{}  {} {} {:?}", sep, i, op.name, op.args))
    }
    s
}

const OPERATIONS: &'static [(u8, (&'static str, u8))] =
    &[(0,("(placeholder)",0)),
      (1,("label",1)),
      (2,("func_info",3)),
      (3,("int_code_end",0)),
      (4,("call",2)),
      (5,("call_last",3)),
      (6,("call_only",2)),
      (7,("call_ext",2)),
      (8,("call_ext_last",3)),
      (9,("bif0",2)),
      (10,("bif1",4)),
      (11,("bif2",5)),
      (12,("allocate",2)),
      (13,("allocate_heap",3)),
      (14,("allocate_zero",2)),
      (15,("allocate_heap_zero",3)),
      (16,("test_heap",2)),
      (17,("init",1)),
      (18,("deallocate",1)),
      (19,("return",0)),
      (20,("send",0)),
      (21,("remove_message",0)),
      (22,("timeout",0)),
      (23,("loop_rec",2)),
      (24,("loop_rec_end",1)),
      (25,("wait",1)),
      (26,("wait_timeout",2)),
      (27,("m_plus",4)),
      (28,("m_minus",4)),
      (29,("m_times",4)),
      (30,("m_div",4)),
      (31,("int_div",4)),
      (32,("int_rem",4)),
      (33,("int_band",4)),
      (34,("int_bor",4)),
      (35,("int_bxor",4)),
      (36,("int_bsl",4)),
      (37,("int_bsr",4)),
      (38,("int_bnot",3)),
      (39,("is_lt",3)),
      (40,("is_ge",3)),
      (41,("is_eq",3)),
      (42,("is_ne",3)),
      (43,("is_eq_exact",3)),
      (44,("is_ne_exact",3)),
      (45,("is_integer",2)),
      (46,("is_float",2)),
      (47,("is_number",2)),
      (48,("is_atom",2)),
      (49,("is_pid",2)),
      (50,("is_reference",2)),
      (51,("is_port",2)),
      (52,("is_nil",2)),
      (53,("is_binary",2)),
      (54,("is_constant",2)),
      (55,("is_list",2)),
      (56,("is_nonempty_list",2)),
      (57,("is_tuple",2)),
      (58,("test_arity",3)),
      (59,("select_val",3)),
      (60,("select_tuple_arity",3)),
      (61,("jump",1)),
      (62,("catch",2)),
      (63,("catch_end",1)),
      (64,("move",2)),
      (65,("get_list",3)),
      (66,("get_tuple_element",3)),
      (67,("set_tuple_element",3)),
      (68,("put_string",3)),
      (69,("put_list",3)),
      (70,("put_tuple",2)),
      (71,("put",1)),
      (72,("badmatch",1)),
      (73,("if_end",0)),
      (74,("case_end",1)),
      (75,("call_fun",1)),
      (76,("make_fun",3)),
      (77,("is_function",2)),
      (78,("call_ext_only",2)),
      (79,("bs_start_match",2)),
      (80,("bs_get_integer",5)),
      (81,("bs_get_float",5)),
      (82,("bs_get_binary",5)),
      (83,("bs_skip_bits",4)),
      (84,("bs_test_tail",2)),
      (85,("bs_save",1)),
      (86,("bs_restore",1)),
      (87,("bs_init",2)),
      (88,("bs_final",2)),
      (89,("bs_put_integer",5)),
      (90,("bs_put_binary",5)),
      (91,("bs_put_float",5)),
      (92,("bs_put_string",2)),
      (93,("bs_need_buf",1)),
      (94,("fclearerror",0)),
      (95,("fcheckerror",1)),
      (96,("fmove",2)),
      (97,("fconv",2)),
      (98,("fadd",4)),
      (99,("fsub",4)),
      (100,("fmul",4)),
      (101,("fdiv",4)),
      (102,("fnegate",3)),
      (103,("make_fun2",1)),
      (104,("try",2)),
      (105,("try_end",1)),
      (106,("try_case",1)),
      (107,("try_case_end",1)),
      (108,("raise",2)),
      (109,("bs_init2",6)),
      (110,("bs_bits_to_bytes",3)),
      (111,("bs_add",5)),
      (112,("apply",1)),
      (113,("apply_last",2)),
      (114,("is_boolean",2)),
      (115,("is_function2",3)),
      (116,("bs_start_match2",5)),
      (117,("bs_get_integer2",7)),
      (118,("bs_get_float2",7)),
      (119,("bs_get_binary2",7)),
      (120,("bs_skip_bits2",5)),
      (121,("bs_test_tail2",3)),
      (122,("bs_save2",2)),
      (123,("bs_restore2",2)),
      (124,("gc_bif1",5)),
      (125,("gc_bif2",6)),
      (126,("bs_final2",2)),
      (127,("bs_bits_to_bytes2",2)),
      (128,("put_literal",2)),
      (129,("is_bitstr",2)),
      (130,("bs_context_to_binary",1)),
      (131,("bs_test_unit",3)),
      (132,("bs_match_string",4)),
      (133,("bs_init_writable",0)),
      (134,("bs_append",8)),
      (135,("bs_private_append",6)),
      (136,("trim",2)),
      (137,("bs_init_bits",6)),
      (138,("bs_get_utf8",5)),
      (139,("bs_skip_utf8",4)),
      (140,("bs_get_utf16",5)),
      (141,("bs_skip_utf16",4)),
      (142,("bs_get_utf32",5)),
      (143,("bs_skip_utf32",4)),
      (144,("bs_utf8_size",3)),
      (145,("bs_put_utf8",3)),
      (146,("bs_utf16_size",3)),
      (147,("bs_put_utf16",3)),
      (148,("bs_put_utf32",3)),
      (149,("on_load",0)),
      (150,("recv_mark",1)),
      (151,("recv_set",1)),
      (152,("gc_bif3",7)),
      (153,("line",1)),
      (154,("put_map_assoc",5)),
      (155,("put_map_exact",5)),
      (156,("is_map",2)),
      (157,("has_map_fields",3)),
      (158,("get_map_elements",3))];

#[derive(Debug)]
struct Op {
    name: &'static str,
    args: Vec<u8>
}

struct CodeOpIterator<'c> {
    i: usize,
    code: &'c [u8]
}

impl<'c> Iterator for CodeOpIterator<'c> {
    type Item = Op;

    fn next(&mut self) -> Option<Op> {
        if self.i >= self.code.len()
            { return None }
        let opcode = self.code[self.i];
        let opdef = OPERATIONS[opcode as usize];
        let opname = (opdef.1).0;
        let nargs = (opdef.1).1 as usize;
        let from = self.i + 1;
        let to = from + nargs;
        self.i = to;
        if to > self.code.len()
            { return None }
        Some (Op { name: opname,
                   args: self.code[from .. to].to_vec() })
    }

}

fn operations(code_chunk: &dream::code::CodeChunk) -> CodeOpIterator {
    CodeOpIterator { i: 0, code: &code_chunk.code }
}
