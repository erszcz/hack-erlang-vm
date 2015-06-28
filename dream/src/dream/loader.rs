use super::{ AtomTable,
             Beam,
             CodeIdx,
             code,
             ExportTable,
             Label };
use super::code::{ ArgTag, BEAMOpcode, CodeChunk };
use std::path::Path;

pub struct State<'a> {
    pub module_name:    &'a str,
    pub beam_file:      Beam,
    pub atoms:          Option<AtomTable>,
    pub code:           Option<Vec<code::Op>>,
    pub labels:         Option<Vec<(Label, CodeIdx)>>,
    pub exports:        Option<ExportTable>
}

impl<'a> State<'a> {

    pub fn new(path: &'a Path) -> Result<State<'a>, Error<'a>> {
        Ok ( State { module_name: try! (module_name(path)),
                     beam_file: try! (Beam::from_file(path)
                                           .or(Err (Error::BeamReadError))),
                     atoms: None,
                     exports: None,
                     code: None,
                     labels: None } )
    }

}

#[derive(Debug)]
pub enum Error<'a> {
    InvalidPath(&'a Path),
    BeamReadError,
    ModuleNameMismatch(/* module: */ String, /* file: */ String),
    ChunkNotFound(&'a str),
    ChunkLoadError,
    LoaderError
}

pub type LoadResult<'a> = Result<(), Error<'a>>;

pub fn load_atoms<'a>(loader: &mut State) -> LoadResult<'a> {
    let ref beam = loader.beam_file;
    let atom_chunk = try! (beam.chunk("Atom")
                               .ok_or(Error::ChunkNotFound("Atom")));
    loader.atoms = Some (AtomTable::from_chunk(atom_chunk));
    Ok (())
}

pub fn check_module_name<'a>(loader: &State) -> LoadResult<'a> {
    let file = loader.module_name;
    if let Some (ref atoms) = loader.atoms {
        let module = try! (atoms.get_atom(0).ok_or(Error::LoaderError));
        if file == module { Ok (()) }
        else {
            Err (Error::ModuleNameMismatch(module, file.to_string()))
        }
    } else {
        Err (Error::LoaderError)
    }
}

pub fn load_code<'a>(loader: &mut State) -> LoadResult<'a> {
    let ref beam = loader.beam_file;
    let chunk = try! (beam.chunk("Code")
                          .ok_or(Error::ChunkNotFound("Code")));
    let code_chunk = try! (CodeChunk::from_chunk(chunk)
                                     .map_err(|_| Error::ChunkLoadError));
    loader.code = Some (code_chunk.code);
    Ok (())
}

pub fn load_labels<'a>(loader: &mut State) -> LoadResult<'a> {
    let mut labels = vec![];
    if let Some (ref code) = loader.code {
        for (i, op) in code.iter().enumerate() {
            try! (load_label(&mut labels, i, op));
        }
        loader.labels = Some (labels);
        Ok (())
    } else {
        Err (Error::LoaderError)
    }
}

fn load_label<'a>(labels: &mut Vec<(Label, CodeIdx)>, i: usize, op: &code::Op)
    -> LoadResult<'a>
{
    if let (BEAMOpcode::label, ref args) = (op.code, &op.args) {
        if let (ArgTag::u, idx) = args[0] {
            labels.push((idx, i as u32));
            Ok (())
        } else {
            Err (Error::LoaderError)
        }
    } else {
        Ok (())
    }
}

pub fn replace_jumps<'a>(loader: &mut State) -> LoadResult<'a> {
    if let (&Some (ref labels), &mut Some (ref mut code)) = (&loader.labels,
                                                             &mut loader.code)
    {
        for op in code.iter_mut()
            { replace_jump(labels, &mut op.args) }
        Ok (())
    } else {
        Err (Error::LoaderError)
    }
}

fn replace_jump(labels: &Vec<(Label, CodeIdx)>, args: &mut [(ArgTag, CodeIdx)]) {
    for &mut (ref tag, ref mut arg) in args.iter_mut() {
        match tag {
            &ArgTag::f if *arg != 0 =>
                *arg = labels[(*arg - 1) as usize].1,
            _ => {}
        }
    }
}

fn module_name(path: &Path) -> Result<&str, Error> {
    path.file_stem()
        .ok_or(Error::InvalidPath (path))
        .and_then(|os_str| os_str.to_str()
                                 .ok_or(Error::InvalidPath (path)))
}
