use std::fs::File;
use std::io::{BufReader, self};
use std::path::Path;
use thiserror::Error;

use crate::mmp::project::ProjectInfo;
use crate::mmp::xml::{ChildNode, self, XMLError};

#[derive(Error, Debug)]
#[error(transparent)]
pub enum XPTPatternError {
    #[error("{0}")]
    XML(#[from] XMLError),

    #[error("{0}")]
    IoError(#[from] io::Error),
    
    #[error("")]
    Invalid,
}

#[derive(Debug)]
pub struct Pattern {
    pub ty: u8,
    pub muted: u8,  // or bool?
    pub name: String,
    pub pos: u16, // check
    pub steps: u8,
    pub notes: Vec<Note>
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
    pub fn from_xml(xml: ChildNode) -> Result<Self, XPTPatternError> {
        let pattern = xml.borrow();

        let steps = pattern.get_attribute("steps")?;
        let ty = pattern.get_attribute("type")?;
        let muted = pattern.get_attribute("muted")?;
        let name = pattern.get_attribute("name")?;
        let pos = pattern.get_attribute("pos")?;

        let mut notes: Vec<Note> = Vec::new();

        for note in pattern.children.iter().map(|x| x.borrow()) {
            notes.push(
                Note {
                    len: note.get_attribute("len")?,
                    key: note.get_attribute("key")?,
                    vol: note.get_attribute("vol")?,
                    pan: note.get_attribute("pan")?,
                    pos: note.get_attribute("pos")?,
                }
            )
        };

        Ok(Self { 
            ty,
            muted,
            name,
            pos,
            steps,
            notes
        })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, XPTPatternError> {
        let a = File::open(path)?;
        let xml_data = xml::build_tree(BufReader::new(a))?;
        let root = xml_data.borrow();

        let project_info = ProjectInfo::new(xml_data.clone())?;

        if project_info.ty != "pattern" {
            return Err(XPTPatternError::Invalid);
        }

        let pattern = root.get_tag("pattern")?;

        Self::from_xml(pattern)
    }
}


#[test]
fn xpt() {
    let xpt = Pattern::from_file("../test/chords.xpt");
    let _ = dbg!(xpt);
}