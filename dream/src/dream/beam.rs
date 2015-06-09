use std::fs::File;
use std::io::Read;
use std::mem;
use std::path::Path;
use std::ptr;

#[derive(Debug)]
pub struct Beam {
    chunks:     Vec<Chunk>
}

#[derive(Debug)]
struct BeamHeader {
    magic:      [u8; 4],
    len:        u32,
    form_type:  [u8; 4]
}

#[derive(Debug)]
struct ChunkHeader {
    id:     [u8; 4],
    len:    u32
}

#[derive(Debug)]
pub struct Chunk {
    pub id:     String,
    pub len:    u32,
    pub data:   Vec<u8>
}

impl Beam {

    pub fn from_file(path: &Path) -> Result<Beam, String> {
        let buf = try!( read_file(path) );
        let header = load_header(&buf);
        try!( check_beam_header(&header) );
        let chunks = load_chunks(&buf, mem::size_of::<BeamHeader>());
        Ok (Beam { chunks: chunks })
    }

    pub fn chunk(&self, name: &str) -> Option<&Chunk> {
        self.chunks.iter().find(|&chunk| chunk.id == name)
    }

}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    let mut b = Vec::new();
    File::open(&path)
        .and_then(|mut f| f.read_to_end(&mut b).and(Ok (b)))
        .or(Err (format!("cannot read {:?}", path)))
}

fn load_header(buf: &Vec<u8>) -> BeamHeader {
    let mut beam_header: BeamHeader = unsafe { mem::uninitialized() };
    unsafe {
        ptr::copy(&buf[0], &mut beam_header as *mut BeamHeader as *mut u8,
                  mem::size_of::<BeamHeader>());
        beam_header.len = u32::from_be(beam_header.len);
    }
    beam_header
}

fn check_beam_header(header: &BeamHeader) -> Result<(), String> {
    let magic = String::from_utf8_lossy(&header.magic);
    if magic != "FOR1"
        { return Err (format!("not a .beam file - invalid magic: {:?}", magic)) }
    let form_type = String::from_utf8_lossy(&header.form_type);
    if form_type != "BEAM"
        { return Err (format!("not a .beam file - invalid form type: {:?}", form_type)) }
    Ok (())
}

fn load_chunks(buf: &Vec<u8>, offset: usize) -> Vec<Chunk> {
    let mut i = offset;
    let mut chunks = vec![];
    while i < buf.len() {
        let (chunk, read) = load_chunk(buf, i);
        i += read;
        chunks.push(chunk);
    }
    chunks
}

// Return chunk and number of bytes read aligned to 4.
fn load_chunk(buf: &Vec<u8>, offset: usize) -> (Chunk, usize) {
    let mut chunk_header: ChunkHeader = unsafe { mem::uninitialized() };
    let header_size = mem::size_of::<ChunkHeader>();
    let data_offset = offset + header_size;
    unsafe {
        ptr::copy(&buf[offset], &mut chunk_header as *mut ChunkHeader as *mut u8,
                  header_size);
    }
    let data_len = u32::from_be(chunk_header.len);
    chunk_header.len = data_len;
    let data = (&buf[data_offset .. data_offset + data_len as usize]).to_vec();
    let chunk_id = String::from_utf8_lossy(&chunk_header.id).into_owned();
    ( Chunk { id: chunk_id,
              len: chunk_header.len,
              data: data },
      round4up(header_size as u32 + data_len) as usize )
}

fn round4up(u: u32) -> u32 {
    // 0x3 is 0b11
    if u & 0x3 == 0 { u }
    // 0xc is 0b1100 so we erase the last two bits.
    else { (u + 4) & 0xfffffffc }
}

#[test]
fn test_load_header() {
    let path = Path::new("../erlang/fac.beam");
    let buf = read_file(path).unwrap();
    let header = load_header(&buf);
    assert_eq!(header.magic, ['F' as u8, 'O' as u8, 'R' as u8, '1' as u8]);
    assert_eq!(header.len, 712);
    assert_eq!(header.form_type, ['B' as u8, 'E' as u8, 'A' as u8, 'M' as u8]);
}

#[test]
fn test_load_chunk() {
    let path = Path::new("../erlang/fac.beam");
    let buf = read_file(path).unwrap();
    let (chunk, read) = load_chunk(&buf, 12);
    assert_eq!(read, 64);
    assert_eq!(chunk.id, "Atom".to_string());
    assert_eq!(chunk.len, 53);
    assert_eq!(buf[12 + read + 0], 'C' as u8);
    assert_eq!(buf[12 + read + 1], 'o' as u8);
}

#[test]
fn test_round4up() {
    assert_eq!(4, round4up(1));
    assert_eq!(56, round4up(0x35));
    assert_eq!(64, round4up(64));
}

#[test]
fn test_loading_beam_from_file() {
    let path = Path::new("../erlang/fac.beam");
    match Beam::from_file(&path) {
        Ok (beam) => {
            assert_eq!("Atom", beam.chunk("Atom").expect("can't get chunk id").id);
            assert_eq!(53, beam.chunk("Atom").expect("can't get chunk len").len);
        },
        Err (e) => panic!(e)
    }
}
