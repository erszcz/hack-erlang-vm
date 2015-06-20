use std;
use super::beam;

#[derive(Debug)]
pub struct CodeChunk {
    pub id:                 String,
    pub len:                u32,

    // `info_fields_len` is at least 16 (4 bytes for each of instruction_set,
    // opcode_max, n_labels, n_functions), though might be more.
    // `code` starts at an offset of `.instruction_set` + `info_fields_len`.
    pub info_fields_len:    u32,

    pub instruction_set:    u32,
    pub opcode_max:         u32,
    pub n_labels:           u32,
    pub n_functions:        u32,

    // Possibly more data here depending on `info_fields_len` value.

    pub code:               Vec<Op>
}

#[derive(Debug)]
pub enum Error {
    // Not a code chunk.
    UnexpectedChunk(String, String),

    // Missing data / malformed chunk.
    InvalidChunk,

    // A module uses higher opcodes than this runtime undertands.
    UnsupportedOpcode(u32),

    InvalidTag
}

fn unexpected_chunk(expected: &str, got: &str) -> Result<CodeChunk, Error> {
    Err ( Error::UnexpectedChunk(expected.to_string(), got.to_string()) )
}

fn unsupported_opcode(opcode: u32) -> Result<CodeChunk, Error> {
    Err ( Error::UnsupportedOpcode(opcode) )
}

impl CodeChunk {

    pub fn from_chunk(chunk: &beam::Chunk) -> Result<CodeChunk, Error> {
        if chunk.id != "Code"
            { return unexpected_chunk("Code", &chunk.id) }
        // Fields from `info_fields_len` to `n_functions` must be present!
        if chunk.data.len() < 5 * std::mem::size_of::<u32>()
            { return Err ( Error::InvalidChunk ) }
        let info_fields_len = u32_from_be(&chunk.data[0..4]);
        let code_start = (/* end of info_fields_len */ 4 +
                          /* offset */ info_fields_len) as usize;
        if code_start >= chunk.data.len()
            { return Err ( Error::InvalidChunk ) }
        let opcode_max = u32_from_be(&chunk.data[8..12]);
        if opcode_max > BEAMOpcode::max_opcode() as u32
            { return unsupported_opcode(opcode_max) }
        let ops = try!(load_bytecode(&chunk.data[code_start..]));
        Ok (CodeChunk {
                id: chunk.id.clone(),
                len: chunk.len,
                info_fields_len: info_fields_len,
                instruction_set: u32_from_be(&chunk.data[4..8]),
                opcode_max: opcode_max,
                n_labels: u32_from_be(&chunk.data[12..16]),
                n_functions: u32_from_be(&chunk.data[16..20]),
                code: ops
        })
    }

}

// TODO:
// - [x] load_bytecode
// - BEAMOpcode name and test for checking that names and codes aren't messed up;
//   use a macro in test to avoid opcode name repetition and learn how
//   to stringify identifiers in macros

fn load_bytecode(bytecode: &[u8]) -> Result<Vec<Op>, Error> {
    let mut i = 0;
    let mut opcodes = vec![];
    while i < bytecode.len() {
        match load_operation(&mut i, bytecode) {
            Ok (op) => opcodes.push(op),
            Err (reason) => return Err (reason)
        }
    }
    Ok (opcodes)
}

fn load_operation(pi: &mut usize,
                  bytecode: &[u8]) -> Result<Op, Error> {
    let i = *pi;
    BEAMOpcode::from_u8(bytecode[i])
               .ok_or(Error::UnsupportedOpcode(bytecode[i] as u32))
               .and_then(|opcode| {
                   load_args(opcode, pi, bytecode)
                   .map(|args| Op { code: opcode, args: args } )
               })
}

fn load_args(opcode: BEAMOpcode, pi: &mut usize,
             bytecode: &[u8]) -> Result<Vec<(OpArg, u32)>, Error> {
    let from = *pi + 1;
    let to = from + opcode.arity() as usize;
    *pi = to;
    if to > bytecode.len()
        { return Err ( Error::InvalidChunk ) }
    transform_args(opcode.arity(), &bytecode[from..to])
}

fn transform_args(nargs: u8, bytes: &[u8]) -> Result<Vec<(OpArg, u32)>, Error> {
    let mut i = 0;
    let mut opargs = vec![];
    while i < bytes.len() {
        match transform_arg(bytes[i], &bytes[i+1..]) {
            Ok ((oparg, nbytes)) => {
                i += nbytes;
                opargs.push(oparg)
            },
            Err (reason) => return Err (reason)
        }
    }
    Ok (opargs)
}

fn transform_arg(arg: u8, rest: &[u8]) -> Result<((OpArg, u32), usize), Error> {
    if let Some (tag) = OpArg::from_u8(arg & 0b111) {
        match tag {
            OpArg::z =>
                panic!("complex terms (aka literals) not supported"),
            OpArg::u | OpArg::i | OpArg::a | OpArg::x | OpArg::y | OpArg::f =>
                value(arg, rest).map(|(v, consumed)| ((tag, v), consumed) )
        }
    } else {
        Err (Error::InvalidTag)
    }
}

fn value(arg: u8, rest: &[u8]) -> Result<(u32, usize), Error> {
    if arg & 0b1000 == 0 {
        Ok ( ((arg >> 4) as u32, 1) )
    } else if arg & 0x10 == 0 {
        let tmp = (arg & 0b1110_0000) as u32;
        let n = (tmp << 3) | rest[0] as u32;
        Ok ( (n, 2) )
    } else {
        Err (Error::InvalidTag)
    }
}

// This is funny.
// Stable Rust doesn't allow to use #![feature(core)]:
//
//    this feature may not be used in the stable release channel
//
// which is needed to access std::raw::Slice (the slice implementation struct).
// This struct is basically a pair of a raw pointer to data and data length.
// This would allow for the following implementation:
//
//     fn u32_from_be(bytes: &[u8]) -> u32 {
//         let bytes: std::raw::Slice<u8> = unsafe { std::mem::transmute(&bytes) };
//         let _u32: *const u32 = bytes.data as *const u32;
//         u32::from_be(unsafe { *_u32 })
//     }
//
fn u32_from_be(bytes: &[u8]) -> u32 {
    if bytes.len() != 4 { panic!("expected 4 bytes") }
    let mut _u32: [u8; 4] = [bytes[0], bytes[1], bytes[2], bytes[3]];
    u32::from_be(unsafe { *(&_u32 as *const u8 as *const u32) })
}

#[derive(Debug)]
pub struct Op {
    pub code: BEAMOpcode,
    pub args: Vec<(OpArg, u32)>
}

impl Op {
    pub fn name(&self) -> &'static str {
        (OPERATIONS[self.code as usize].1).0
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum BEAMOpcode {
    label               = 1,
    func_info           = 2,
    int_code_end        = 3,
    call                = 4,
    call_only           = 6,
    allocate_zero       = 14,
    test_heap           = 16,
    deallocate          = 18,
    return_             = 19,
    is_eq_exact         = 43,
    is_tuple            = 57,
    test_arity          = 58,
    move_               = 64,
    get_tuple_element   = 66,
    put_tuple           = 70,
    put                 = 71,
    call_ext_only       = 78,
    gc_bif2             = 125,
    line                = 153
}

impl BEAMOpcode {

    fn from_u8(code: u8) -> Option<BEAMOpcode> {
        match code {
            1   => Some ( BEAMOpcode::label ),
            2   => Some ( BEAMOpcode::func_info ),
            3   => Some ( BEAMOpcode::int_code_end ),
            4   => Some ( BEAMOpcode::call ),
            6   => Some ( BEAMOpcode::call_only ),
            14  => Some ( BEAMOpcode::allocate_zero ),
            16  => Some ( BEAMOpcode::test_heap ),
            18  => Some ( BEAMOpcode::deallocate ),
            19  => Some ( BEAMOpcode::return_ ),
            43  => Some ( BEAMOpcode::is_eq_exact ),
            57  => Some ( BEAMOpcode::is_tuple ),
            58  => Some ( BEAMOpcode::test_arity ),
            64  => Some ( BEAMOpcode::move_ ),
            66  => Some ( BEAMOpcode::get_tuple_element ),
            70  => Some ( BEAMOpcode::put_tuple ),
            71  => Some ( BEAMOpcode::put ),
            78  => Some ( BEAMOpcode::call_ext_only ),
            125 => Some ( BEAMOpcode::gc_bif2 ),
            153 => Some ( BEAMOpcode::line ),
            _   => None
        }
    }

    fn arity(self) -> u8 {
        match self {
            BEAMOpcode::label             => 1,
            BEAMOpcode::func_info         => 3,
            BEAMOpcode::int_code_end      => 0,
            BEAMOpcode::call              => 2,
            BEAMOpcode::call_only         => 2,
            BEAMOpcode::allocate_zero     => 2,
            BEAMOpcode::test_heap         => 2,
            BEAMOpcode::deallocate        => 1,
            BEAMOpcode::return_           => 0,
            BEAMOpcode::is_eq_exact       => 3,
            BEAMOpcode::is_tuple          => 2,
            BEAMOpcode::test_arity        => 3,
            BEAMOpcode::move_             => 2,
            BEAMOpcode::get_tuple_element => 3,
            BEAMOpcode::put_tuple         => 2,
            BEAMOpcode::put               => 1,
            BEAMOpcode::call_ext_only     => 2,
            BEAMOpcode::gc_bif2           => 6,
            BEAMOpcode::line              => 1
        }
    }

    // Hacky, but this way this function doesn't have to updated
    // each time a new opcode is added.
    fn max_opcode() -> u8 {
        let mut code = 255;
        while code > 0 {
            match BEAMOpcode::from_u8(code) {
                None => { code -= 1; continue },
                _ => break
            }
        }
        assert!(code > 0, "max valid opcode cannot be 0!");
        code
    }

}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum OpArg {
    u,
    i,
    a,
    x,
    y,
    f,
    z
}

impl OpArg {

    fn from_u8(tag: u8) -> Option<OpArg> {
        match tag {
            0 => Some (OpArg::u),
            1 => Some (OpArg::i),
            2 => Some (OpArg::a),
            3 => Some (OpArg::x),
            4 => Some (OpArg::y),
            5 => Some (OpArg::f),
            7 => Some (OpArg::z),
            _ => None
        }
    }

}

#[test]
fn test_max_opcode() {
    assert_eq!( 153, BEAMOpcode::max_opcode() );
}

trait BEAMOps<T> {
    fn call_ext_only()      -> T;
    fn call_only()          -> T;
    fn func_info()          -> T;
    fn gc_bif2()            -> T;
    fn get_tuple_element()  -> T;
    fn int_code_end()       -> T;
    fn is_eq_exact()        -> T;
    fn is_tuple()           -> T;
    fn label()              -> T;
    fn line()               -> T;
    fn move_()              -> T;
    fn put()                -> T;
    fn put_tuple()          -> T;
    fn return_()            -> T;
    fn test_arity()         -> T;
    fn test_heap()          -> T;
}

pub const OPERATIONS: &'static [(u8, (&'static str, u8))] =
    &[(0,("(invalid opcode)",0)),
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
