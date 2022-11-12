use crate::mmp::xml::{Node, XMLError};

#[derive(Debug)]
pub struct ProjectInfo {
    pub ty: String,
    pub creator: String,
    pub version: usize, // NOTE: Older versions uses floats
    pub creator_version: String,
}

impl ProjectInfo {
    pub fn new(info: &Node) -> Result<Self, XMLError> {
        if info.tag() != "lmms-project" {
            return Err(XMLError::Error("invalid LMMS format, expected lmms-project".into()));
        }

        let ty = info.get_attribute("type")?;
        let creator = info.get_attribute("creator")?;
        let version = info.get_attribute("version")?;
        let creator_version = info.get_attribute("creatorversion")?;
        
        Ok(Self {
            ty,
            creator,
            version,
            creator_version,
        })
    }
}