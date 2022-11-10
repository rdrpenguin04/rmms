mod xml;
use std::{path::Path, fs::File};
use std::io::{self, BufReader};
use thiserror::Error;
use xml::{ChildNode, XMLError};

#[derive(Error, Debug)]
pub enum MMPParseError {
    #[error("{0}")]
    Invalid(String),

    #[error("{0}")]
    XML(#[from] XMLError),

    #[error("{0}")]
    IoError(#[from] io::Error),
}

#[derive(Debug)]
pub struct MMP {
    header: Header,
    song_info: SongInfo,
    creator: String,
    version: usize,
    creator_version: String,
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
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, MMPParseError>{
        let a = File::open(path)?;
        let xml_data = xml::build_tree(BufReader::new(a))?;
        let root = xml_data.borrow();

        if root.tag() != "lmms-project" {
            return Err(MMPParseError::Invalid("()".into()));
        }
        let mmp_type: String = root.get_attribute("type")?;

        if mmp_type != "song" {
            return Err(MMPParseError::Invalid("()".into()));
        }
        
        let header = Header::new(root.get_tag("head")?)?;
        let song_info = SongInfo::new(root.get_tag("song")?)?;

        let creator = root.get_attribute("creator")?;
        let version = root.get_attribute("version")?;
        let creator_version = root.get_attribute("creatorversion")?;
        
        Ok(Self {
            header,
            song_info,
            creator,
            creator_version,
            version,
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
                head.get_attribute("timesig_denominator")?
            ), 
            master_pitch: head.get_attribute("masterpitch")?
        })
    }
}

impl SongInfo {
    pub fn new(xml: ChildNode) -> Result<Self, MMPParseError> {
        let song_info = xml.borrow();
        println!("Tags in song:");

        for attr in &song_info.children {
            println!("- {}",attr.borrow().tag());
        }

        Ok(Self)
    }
}

#[test]
fn test() {
    let mmp = MMP::load("../test/format.mmp");
    let _ = dbg!(mmp);
}