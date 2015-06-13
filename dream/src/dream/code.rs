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

    pub code:               Vec<u8>
}

#[derive(Debug)]
pub enum Error {
    UnexpectedChunk(String, String),
    InvalidChunk
}

fn unexpected_chunk(expected: &str, got: &str) -> Result<CodeChunk, Error> {
    Err ( Error::UnexpectedChunk(expected.to_string(), got.to_string()) )
}

impl CodeChunk {

    pub fn from_chunk(chunk: &beam::Chunk) -> Result<CodeChunk, Error> {
        if chunk.id != "Code"
            { return unexpected_chunk("Code", &chunk.id) }
        // Fields from `info_fields_len` to `n_functions` must be present!
        if chunk.data.len() < 5 * std::mem::size_of::<u32>()
            { return Err ( Error::InvalidChunk ) }
        Ok (CodeChunk {
                id: chunk.id.clone(),
                len: chunk.len,
                info_fields_len: u32_from_be(&chunk.data[0..4]),
                instruction_set: u32_from_be(&chunk.data[4..8]),
                opcode_max: u32_from_be(&chunk.data[8..12]),
                n_labels: u32_from_be(&chunk.data[12..16]),
                n_functions: u32_from_be(&chunk.data[16..20]),
                code: chunk.data[20..].to_vec()
        })
    }

    pub fn operations(&self) -> CodeOpIterator {
        CodeOpIterator { i: 0, code: &self.code }
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
    pub name: &'static str,
    pub args: Vec<u8>
}

pub struct CodeOpIterator<'c> {
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

// TODO: this is just a subset of BEAM opcodes!
#[derive(Debug)]
enum OpType {
    CallExtOnly     (u8, u8),
    CallOnly        (u8, u8),
    FuncInfo        (u8, u8, u8),
    GcBif2          (u8, u8, u8, u8, u8, u8),
    GetTupleElement (u8, u8, u8),
    IntCodeEnd,
    IsEqExact       (u8, u8, u8),
    IsTuple         (u8, u8),
    Label           (u8),
    Line            (u8),
    Move            (u8, u8),
    Put             (u8),
    PutTuple        (u8, u8),
    Return,
    TestArity       (u8, u8, u8),
    TestHeap        (u8, u8)

      invalid_opcode",0)),
      label",1)),
      func_info",3)),
      int_code_end",0)),
      call",2)),
      call_last",3)),
      call_only",2)),
      call_ext",2)),
      call_ext_last",3)),
      bif0",2)),
      bif1",4)),
      bif2",5)),
      allocate",2)),
      allocate_heap",3)),
      allocate_zero",2)),
      allocate_heap_zero",3)),
      test_heap",2)),
      init",1)),
      deallocate",1)),
      return",0)),
      send",0)),
      remove_message",0)),
      timeout",0)),
      loop_rec",2)),
      loop_rec_end",1)),
      wait",1)),
      wait_timeout",2)),
      m_plus",4)),
      m_minus",4)),
      m_times",4)),
      m_div",4)),
      int_div",4)),
      int_rem",4)),
      int_band",4)),
      int_bor",4)),
      int_bxor",4)),
      int_bsl",4)),
      int_bsr",4)),
      int_bnot",3)),
      is_lt",3)),
      is_ge",3)),
      is_eq",3)),
      is_ne",3)),
      is_eq_exact",3)),
      is_ne_exact",3)),
      is_integer",2)),
      is_float",2)),
      is_number",2)),
      is_atom",2)),
      is_pid",2)),
      is_reference",2)),
      is_port",2)),
      is_nil",2)),
      is_binary",2)),
      is_constant",2)),
      is_list",2)),
      is_nonempty_list",2)),
      is_tuple",2)),
      test_arity",3)),
      select_val",3)),
      select_tuple_arity",3)),
      jump",1)),
      catch",2)),
      catch_end",1)),
      move",2)),
      get_list",3)),
      get_tuple_element",3)),
      set_tuple_element",3)),
      put_string",3)),
      put_list",3)),
      put_tuple",2)),
      put",1)),
      badmatch",1)),
      if_end",0)),
      case_end",1)),
      call_fun",1)),
      make_fun",3)),
      is_function",2)),
      call_ext_only",2)),
      bs_start_match",2)),
      bs_get_integer",5)),
      bs_get_float",5)),
      bs_get_binary",5)),
      bs_skip_bits",4)),
      bs_test_tail",2)),
      bs_save",1)),
      bs_restore",1)),
      bs_init",2)),
      bs_final",2)),
      bs_put_integer",5)),
      bs_put_binary",5)),
      bs_put_float",5)),
      bs_put_string",2)),
      bs_need_buf",1)),
      fclearerror",0)),
      fcheckerror",1)),
      fmove",2)),
      fconv",2)),
      fadd",4)),
      fsub",4)),
      fmul",4)),
      fdiv",4)),
      fnegate",3)),
      make_fun2",1)),
      try",2)),
      try_end",1)),
      try_case",1)),
      try_case_end",1)),
      raise",2)),
      bs_init2",6)),
      bs_bits_to_bytes",3)),
      bs_add",5)),
      apply",1)),
      apply_last",2)),
      is_boolean",2)),
      is_function2",3)),
      bs_start_match2",5)),
      bs_get_integer2",7)),
      bs_get_float2",7)),
      bs_get_binary2",7)),
      bs_skip_bits2",5)),
      bs_test_tail2",3)),
      bs_save2",2)),
      bs_restore2",2)),
      gc_bif1",5)),
      gc_bif2",6)),
      bs_final2",2)),
      bs_bits_to_bytes2",2)),
      put_literal",2)),
      is_bitstr",2)),
      bs_context_to_binary",1)),
      bs_test_unit",3)),
      bs_match_string",4)),
      bs_init_writable",0)),
      bs_append",8)),
      bs_private_append",6)),
      trim",2)),
      bs_init_bits",6)),
      bs_get_utf8",5)),
      bs_skip_utf8",4)),
      bs_get_utf16",5)),
      bs_skip_utf16",4)),
      bs_get_utf32",5)),
      bs_skip_utf32",4)),
      bs_utf8_size",3)),
      bs_put_utf8",3)),
      bs_utf16_size",3)),
      bs_put_utf16",3)),
      bs_put_utf32",3)),
      on_load",0)),
      recv_mark",1)),
      recv_set",1)),
      gc_bif3",7)),
      line",1)),
      put_map_assoc",5)),
      put_map_exact",5)),
      is_map",2)),
      has_map_fields",3)),
      get_map_elements",3))];

}

impl OpType {

    fn code(self) -> usize {
        match self {
            CallExtOnly (_, _)        => 78,
            CallOnly (_, _)           => 6,
            FuncInfo (_, _, _)        => 2,
            GcBif2 (_, _, _, _, _, _) => 125,
            GetTupleElement (_, _, _) => 66,
            IntCodeEnd                => 3,
            IsEqExact (_, _, _)       => 43,
            IsTuple (_, _)            => 57,
            Label (_)                 => 1,
            Line (_)                  => 153,
            Move (_, _)               => 64,
            Put (_)                   => 71,
            PutTuple (_, _)           => 70,
            Return                    => 19,
            TestArity (_, _, _)       => 58,
            TestHeap (_, _)           => 16
        }
    }

}

pub const OPERATIONS: &'static [(u8, (&'static str, u8))] =
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

#[[test]
fn op_type_index_test() {
    assert_eq!(
    CallExtOnly (0, 0)
        CallOnly (0, 0)
        FuncInfo (0, 0, 0)
        GcBif2 (0, 0, 0, 0, 0, 0)
        GetTupleElement (0, 0, 0)
        IntCodeEnd
        IsEqExact (0, 0, 0)
        IsTuple (0, 0)
        Label (0)
        Line (0)
        Move (0, 0)
        Put (0)
        PutTuple (0, 0)
        Return
        TestArity (0, 0, 0)
        TestHeap (0, 0)
}
