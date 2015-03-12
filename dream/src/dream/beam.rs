use std::fs::File;
use std::io::{ BufRead, BufReader };
use std::mem;
use std::path::Path;

struct BeamHeader {
    magic:      [u8; 4],
    len:        u32,
    form_type:  [u8; 4]
}

fn read_beam(path: &Path) -> Result<(), String> {
    let f = match File::open(&path) {
        Err (e) => panic!(e),
        Ok (mut f) => f
    };
    let mut reader = BufReader::with_capacity(12, f);
    let b = match reader.fill_buf() {
        Err (e) => panic!(e),
        Ok (b) => b
    };
    let h: BeamHeader = BeamHeader { magic: [b[0], b[1], b[2], b[3]],
                                     len: unsafe { mem::transmute_copy(&b[4]) },
                                     form_type: [b[8], b[9], b[10], b[11]] };
    if !(h.magic == ['F' as u8, 'O' as u8, 'R' as u8, '1' as u8])
        { panic!("magic: {:?}", h.magic) }
    if !(h.form_type == ['B' as u8, 'E' as u8, 'A' as u8, 'M' as u8])
        { panic!("form_type: {:?}", h.form_type) }
    return Ok (())
}

#[test]
fn test_read_beam() {
    let path = Path::new("../erlang/fac.beam");
    read_beam(&path);
}
