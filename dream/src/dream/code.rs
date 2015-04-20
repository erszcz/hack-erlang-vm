use std;
use super::beam;

type Opcode = u32;

pub struct CodeTable {
    opcodes: Vec<Opcode>
}

const NULL_OPCODE: Opcode = 0;

impl CodeTable {

    pub fn from_chunk(chunk: &beam::Chunk) -> CodeTable {
        let _max_opcode_offset = 8;
        let _no_of_labels_offset = 12;
        let _no_of_exports_offset = 16;
        let _offset = 20;
        let mut ct = CodeTable::new();
        for opcode in chunk.data.chunks(4) {
            if opcode.len() < 4
                // Last incomplete chunk.
                { continue }
            unsafe {
                ct.opcodes.push(*(&opcode[0] as *const u8 as *const Opcode));
            }
        }
        ct
    }

    fn new() -> CodeTable {
        let opcodes = vec![NULL_OPCODE];
        CodeTable { opcodes: opcodes }
    }

    pub fn list(&self) -> Vec<(u8,u8,u8,u8)> {
        self.opcodes.iter()
            .map(|&opcode| unsafe {
                let as_slice = std::slice::from_raw_parts(opcode as *const u32
                                                          as *const u8, 4);
                (as_slice[0], as_slice[1], as_slice[2], as_slice[3])
            })
            .collect()
    }

}

#[test]
fn test_listing_opcodes() {
    panic!("not implemented yet");
}
