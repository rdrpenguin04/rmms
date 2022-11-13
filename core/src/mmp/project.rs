use crate::mmp::xml::{self, Node};

#[derive(Debug)]
pub struct Info {
    pub ty: String,
    pub creator: String,
    pub version: usize, // NOTE: Older versions uses floats
    pub creator_version: String,
}

impl Info {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(info: &Node) -> xml::Result<Self> {
        if info.tag() != "lmms-project" {
            return Err(xml::Error::Error(
                "invalid LMMS format, expected lmms-project".into(),
            ));
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
