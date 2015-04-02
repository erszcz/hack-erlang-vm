use std;
use std::collections::HashMap;
use std::fmt::{ Debug, Formatter };
use std::fs::File;
use std::io::Error as IOError;
use std::io::{ BufRead, BufReader };
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

//impl Debug for BeamHeader {
//    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
//        writeln!(formatter,
//                 "BeamHeader:\n  magic: {:?})\n  len: {:?}\n  form_type: {:?}",
//                 self.magic, self.len, self.form_type)
//    }
//}

#[derive(Debug)]
struct ChunkHeader {
    chunk_id:   [u8; 4],
    len:        u32
}

struct Chunk {
    chunk_id:   String,
    data:       Vec<u8>
}

impl Beam {

    pub fn load(path: &Path) -> Result<(), String> {
        let f = match File::open(&path) {
            Err (e) => panic!(e),
            Ok (mut f) => f
        };
        let (header, file) = load_header(f);
        println!("{:?}", header);
        assert_eq!(712, header.len);
        let left_to_read = header.len as usize - 4;
        let mut reader = BufReader::with_capacity(left_to_read, file);
        let chunks = load_chunks(reader);
        Ok (())
    }

}

fn load_header(f: File) -> (BeamHeader, File) {
    let sz = mem::size_of::<BeamHeader>();
    let mut reader = BufReader::with_capacity(sz, f);
    let b: Vec<u8> = match reader.fill_buf() {
        Err (e) => panic!(e),
        Ok (b) => b.iter().cloned().collect()
    };
    reader.consume(sz);
    let h = BeamHeader { magic: [b[0], b[1], b[2], b[3]],
                         len: unsafe { let d: u32 = mem::transmute_copy(&b[4]);
                                       Int::from_be(d) },
                         form_type: [b[8], b[9], b[10], b[11]] };
    if !(h.magic == ['F' as u8, 'O' as u8, 'R' as u8, '1' as u8])
        { panic!("magic: {:?}", h.magic) }
    if !(h.form_type == ['B' as u8, 'E' as u8, 'A' as u8, 'M' as u8])
        { panic!("form_type: {:?}", h.form_type) }
    let f = reader.into_inner();
    (h, f)
}

fn load_chunks(mut reader: BufReader<File>) -> HashMap<String, Chunk> {
    let mut chunks: HashMap<String, Chunk> = HashMap::new();
    let data: Vec<u8> = match reader.fill_buf() {
        Err (e) => panic!(e),
        Ok (b) => b.iter().cloned().collect()
    };
    let sz = mem::size_of::<ChunkHeader>();
    let mut i = 0;
    while i < data.len() {
        let mut ch: ChunkHeader = unsafe { mem::uninitialized() };
        unsafe {
            ptr::copy(&mut ch as *mut ChunkHeader as *mut u8,
                      data.index(&i), sz);
        }
        ch.len = Int::from_be(ch.len);
        println!("{:?}", ch);
        assert_eq!(['A' as u8, 't' as u8, 'o' as u8, 'm' as u8], ch.chunk_id);
        assert_eq!(53, ch.len);
        i = 1000;
    }
    chunks
}

fn load_chunk(f: File) -> Option<(Chunk, File)> {
    let sz = mem::size_of::<ChunkHeader>();
    let mut hreader = BufReader::with_capacity(sz, f);
    let b: Vec<u8> = match hreader.fill_buf() {
        Err (e) => panic!(e),
        Ok (b) => {
            if b.is_empty()
                { return None }
            else
                { b.iter().cloned().collect() }
        }
    };
    hreader.consume(sz);
    let h = ChunkHeader {
        chunk_id: [b[0], b[1], b[2], b[3]],
        len: round4up(unsafe { mem::transmute_copy(&b[4]) })
    };
    Some (( Chunk { chunk_id: "ala".to_string(),
                    data: vec![] },
            hreader.into_inner() ))
}

fn round4up(u: u32) -> u32 {
    // 0xc is 0b1100 so we erase the last two bits.
    (u + 4) & 0xfffffffc
}

#[test]
fn test_loading_beam() {
    let path = Path::new("../erlang/fac.beam");
    Beam::load(&path);
}

#[test]
fn test_round4up() {
    assert_eq!(4, round4up(1));
    assert_eq!(56, round4up(0x35));
}

//pub fn from_file(path: &Path) -> Result<Beam, IOError> {
//    match File::open(path) {
//        Ok (beam) => Some (Beam { loaded_from: &(*path).clone(),
//                                  chunks: read_chunks(beam) }),
//    }
//}

//fn read_chunks(f: File) -> HashMap<String, Vec<u8>> {
//    HashMap::new()
//}
