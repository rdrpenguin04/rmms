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
    
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug)]
pub struct Pattern {
    pub project_info: Option<ProjectInfo>,
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
    pub fn with_project_info(mut self, project_info: ProjectInfo) -> Self {
        self.project_info = Some(project_info);
        self
    }

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
            notes,
            project_info: None,
        })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, XPTPatternError> {
        let file = File::open(path)?;
        let root = xml::build_tree(BufReader::new(file))?;
        let project_info = ProjectInfo::new(&root)?;

        if project_info.ty != "pattern" {
            return Err(XPTPatternError::Invalid("not an LMMS pattern file".into()));
        }

        let pattern = root.get_tag("pattern")?;

        Ok(Self::from_xml(pattern)?
            .with_project_info(project_info))
    }
}


#[test]
fn xpt() {
    let xpt = Pattern::from_file("../test/chords.xpt");
    if let Err(e) = xpt {
        println!("{}", e);
    } else {
        dbg!(xpt);
    };
}