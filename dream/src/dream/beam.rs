use std;
use std::collections::HashMap;
use std::fmt::{ Debug, Formatter };
use std::fs::File;
use std::io::Error as IOError;
use std::io::Read;
use std::num::Int;
use std::mem;
use std::ops::Index;
use std::path::Path;
use std::ptr;

struct Beam;

#[derive(Debug)]
struct BeamHeader {
    magic:      [u8; 4],
    len:        u32,
    form_type:  [u8; 4]
}

#[derive(Debug)]
struct ChunkHeader {
    chunk_id:   [u8; 4],
    len:        u32
}

#[derive(Debug)]
struct Chunk {
    chunk_id:   String,
    len:        u32,
    data:       Vec<u8>
}

impl Beam {

    pub fn load(path: &Path) -> Result<(), String> {
        let buf = read_file(path);
        let beam_header = load_header(&buf);
        let chunks = load_chunks(&buf, mem::size_of::<BeamHeader>());
        Ok (())
    }

}

fn read_file(path: &Path) -> Vec<u8> {
    let mut f = match File::open(&path) {
        Err (e) => panic!(e),
        Ok (mut f) => f
    };
    let mut b = Vec::new();
    f.read_to_end(&mut b);
    b
}

fn load_header(buf: &Vec<u8>) -> BeamHeader {
    let mut beam_header: BeamHeader = unsafe { mem::uninitialized() };
    unsafe {
        ptr::copy(&mut beam_header as *mut BeamHeader as *mut u8,
                  buf.index(&0), mem::size_of::<BeamHeader>());
        beam_header.len = Int::from_be(beam_header.len);
    }
    beam_header
}

fn load_chunks(buf: &Vec<u8>, offset: usize) -> Vec<Chunk> {
    let mut i = offset;
    while i < buf.len() {
        i += 1;
    }
    vec![]
}

#[test]
fn test_load_header() {
    let path = Path::new("../erlang/fac.beam");
    let buf = read_file(path);
    let header = load_header(&buf);
    assert_eq!(header.magic, ['F' as u8, 'O' as u8, 'R' as u8, '1' as u8]);
    assert_eq!(header.len, 712);
    assert_eq!(header.form_type, ['B' as u8, 'E' as u8, 'A' as u8, 'M' as u8]);
}

#[test]
fn test_loading_beam() {
    let path = Path::new("../erlang/fac.beam");
    Beam::load(&path);
}
