mod project;
mod xml;
mod xpt;
mod zlib;

use project::ProjectInfo;
use std::io::{self, BufRead, BufReader, Seek};
use std::{fs::File, path::Path};
use thiserror::Error;
use xml::{ChildNode, Node, XMLError};
use xpt::{Pattern, XPTPatternError};

#[derive(Error, Debug)]
pub enum MMPParseError {
    #[error("{0}")]
    Invalid(String),

    #[error("{0}")]
    XML(#[from] XMLError),

    #[error("{0}")]
    IoError(#[from] io::Error),

    #[error("{0}")]
    XPTError(#[from] XPTPatternError),
}

#[derive(Debug)]
pub struct MMP {
    project_info: ProjectInfo,
    header: Header,
    song_info: SongInfo,
}

#[derive(Debug)]
pub struct Header {
    bpm: f32,
    vol: f32,
    time_sig: (u8, u8),
    master_pitch: i8,
}

#[derive(Debug)]
pub struct SongInfo;

impl MMP {
    /// Load LMMS project from path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, MMPParseError> {
        let Some(ext) = &path.as_ref().extension() else {
            return Err(MMPParseError::Invalid("File extension required".into()))
        };

        let file = BufReader::new(File::open(&path)?);

        match ext.to_str() {
            Some("mmp") | Some("MMP") => Self::load_mmp(file),
            Some("mmpz") | Some("MMPZ") => Self::load_mmpz(file),
            _ => Err(MMPParseError::Invalid(
                "Expected extension mmp or mmpz".into(),
            )),
        }
    }

    /// Load LMMS project from reader
    pub fn load_mmp<R: BufRead + Seek>(file: R) -> Result<Self, MMPParseError> {
        Self::parse_mmp(xml::build_tree(file)?)
    }

    /// Load compressed LMMS project from reader
    pub fn load_mmpz<R: BufRead + Seek>(file: R) -> Result<Self, MMPParseError> {
        Self::parse_mmp(zlib::decompress(file)?)
    }

    fn parse_mmp(root: Node) -> Result<Self, MMPParseError> {
        let project_info = ProjectInfo::new(&root)?;

        if project_info.ty != "song" {
            return Err(MMPParseError::Invalid("not an LMMS project file".into()));
        }

        let header = Header::new(root.get_tag("head")?)?;
        let song_info = SongInfo::new(root.get_tag("song")?)?;

        Ok(Self {
            header,
            project_info,
            song_info,
        })
    }
}

impl Header {
    pub fn new(xml: ChildNode) -> Result<Self, MMPParseError> {
        let head = xml.borrow();

        Ok(Self {
            bpm: head.get_attribute("bpm")?,
            vol: head.get_attribute("mastervol")?,
            time_sig: (
                head.get_attribute("timesig_numerator")?,
                head.get_attribute("timesig_denominator")?,
            ),
            master_pitch: head.get_attribute("masterpitch")?,
        })
    }
}

impl SongInfo {
    pub fn new(xml: ChildNode) -> Result<Self, MMPParseError> {
        let song_info = xml.borrow();
        println!("Tags in song:");

        for attr in &song_info.children {
            println!("- {}", attr.borrow().tag());
        }

        Ok(Self)
    }
}

#[test]
fn test() {
    let mmp = MMP::load("../test/format.mmpz");
    if let Err(e) = mmp {
        println!("{}", e);
    } else {
        dbg!(mmp);
    };
}
