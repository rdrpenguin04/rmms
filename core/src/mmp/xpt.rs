use crate::mmp::project::ProjectInfo;
use crate::mmp::xml::{self, ChildNode, Node};
use crate::mmp::zlib;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek};
use std::path::Path;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum Error {
    #[error("{0}")]
    Xml(#[from] xml::Error),

    #[error("{0}")]
    IoError(#[from] io::Error),

    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug)]
pub struct Pattern {
    pub ty: u8,
    pub muted: u8, // or bool?
    pub name: String,
    pub pos: u16, // check
    pub steps: u8,
    pub notes: Vec<Note>,
}

#[derive(Debug)]
pub struct Note {
    pub len: u8,
    pub key: u8,
    pub vol: u8,
    pub pan: u8,
    pub pos: u16, // check
}

impl Pattern {
    /// Load patten from path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let Some(ext) = &path.as_ref().extension() else {
            return Err(Error::Invalid("File extension required".into()))
        };

        let file = BufReader::new(File::open(&path)?);

        match ext.to_str() {
            Some("xpt") | Some("XPT") => Self::load_xpt(file),
            Some("xptz") | Some("XPTZ") => Self::load_xptz(file),
            _ => Err(Error::Invalid("Expected xpt or xptz".into())),
        }
    }

    /// load pattern from reader
    pub fn load_xpt<R: BufRead + Seek>(file: R) -> Result<Self> {
        Self::parse_xpt(xml::build_tree(file)?)
    }

    /// Load compressed pattern from reader
    pub fn load_xptz<R: BufRead + Seek>(file: R) -> Result<Self> {
        Self::parse_xpt(zlib::decompress(file)?)
    }

    /// Validate (xpt|xptz) XML node.
    fn parse_xpt(root: Node) -> Result<Self> {
        let project_info = ProjectInfo::new(&root)?;

        if project_info.ty != "pattern" {
            return Err(Error::Invalid("not an LMMS pattern file".into()));
        }

        Self::from_xml(root.get_tag("pattern")?)
    }

    /// LMMS' pattern data in mmp/mmpz is identical to the xpt format, but without the "lmms-project" tag.
    ///
    /// This function allows the MMP struct to use this.
    pub fn from_xml(xml: ChildNode) -> Result<Self> {
        let pattern = xml.borrow();

        let steps = pattern.get_attribute("steps")?;
        let ty = pattern.get_attribute("type")?;
        let muted = pattern.get_attribute("muted")?;
        let name = pattern.get_attribute("name")?;
        let pos = pattern.get_attribute("pos")?;

        let mut notes: Vec<Note> = Vec::new();

        for note in pattern.children.iter().map(|x| x.borrow()) {
            notes.push(Note {
                len: note.get_attribute("len")?,
                key: note.get_attribute("key")?,
                vol: note.get_attribute("vol")?,
                pan: note.get_attribute("pan")?,
                pos: note.get_attribute("pos")?,
            })
        }

        Ok(Self {
            ty,
            muted,
            name,
            pos,
            steps,
            notes,
        })
    }
}

#[test]
fn xpt() {
    let xpt = Pattern::load("../test/chords.xptz");
    if let Err(e) = xpt {
        println!("{}", e);
    } else {
        dbg!(xpt);
    };
}
