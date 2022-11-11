use crate::mmp::xml::{ChildNode, XMLError};

#[derive(Debug)]
pub struct ProjectInfo {
    pub r#type: String,
    pub creator: String,
    pub version: usize,
    pub creator_version: String,
}

impl ProjectInfo {
    pub fn new(xml: ChildNode) -> Result<Self, XMLError> {
        let info = xml.borrow();
        
        if info.tag() != "lmms-project" {
            return Err(XMLError::Error("Invalid LMMS format, expected lmms-project".into()));
        }

        let r#type = info.get_attribute("type")?;
        let creator = info.get_attribute("creator")?;
        let version = info.get_attribute("version")?;
        let creator_version = info.get_attribute("creatorversion")?;
        
        Ok(Self {
            r#type,
            creator,
            version,
            creator_version,
        })
    }
}