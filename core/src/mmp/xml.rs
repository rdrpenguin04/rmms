use quick_xml::{self, Reader, events::Event, Error as QuickXMLError};
use std::borrow::Cow;
use std::io::{BufRead, Seek};
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::str::{from_utf8, FromStr};
use thiserror::Error;

pub type ParentNode = Weak<RefCell<Node>>;
pub type ChildNode= Rc<RefCell<Node>>;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum XMLError{
    #[error("Invalid XML: {0}")]
    Invalid(String),

    #[error("Could not find attribute \"{0}\"")]
    AttrNotPresent(String),

    #[error("Could not find tag \"{0}\"")]
    TagNotPresent(String),
    
    #[error("{0}")]
    Error(String),

    #[error("{0}")]
    ParseError(#[from] QuickXMLError),

    #[error("{0}")]
    TypeCoercionError(#[from] TypeCoercionError)
}

impl XMLError {
    pub fn invalid(err: &str) -> Self {
        Self::Invalid(err.to_owned())
    }

    pub fn attr_not_present(err: &str) -> Self {
        Self::AttrNotPresent(err.to_owned())
    }

    pub fn tag_not_present(err: &str) -> Self {
        Self::TagNotPresent(err.to_owned())
    }
}

#[derive(Error, Debug)]
#[error("Could not convert \"{0}\" to the desired type, {1}.")]
pub struct TypeCoercionError(String, Box<dyn std::error::Error + Send + Sync>);

impl TypeCoercionError {
    pub fn new<E>(input: &str, error: E) -> Self 
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self(input.to_owned(), error.into())
    }
}

/// Helper function to convert strings to different types
fn convert<T>(str: &str) -> Result<T, TypeCoercionError>
where
    T: FromStr + Default, 
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static
{
    T::from_str(str)
        .map_err(|e| TypeCoercionError::new(str, e))
}

/// XML node
#[derive(Default, Debug)]
pub struct Node {
    parent: Option<ParentNode>,
    pub children: Vec<ChildNode>,
    raw_tag: Vec<u8>,
    attributes: Vec<(Vec<u8>, Vec<u8>)>,
    _cdata: Option<String>,
}

impl Node {
    /// Current tag of element
    pub fn tag(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.raw_tag)
    }

    pub fn attributes(&self) -> Vec<(Cow<str>, Cow<str>)> {
        self.attributes
            .iter()
            .map(|(k,v)| 
                (String::from_utf8_lossy(k), String::from_utf8_lossy(v))
            )
            .collect()
    }

    /// Recursively traverse tree to find a desired tag
    pub fn get_tag(&self, tag: &str) -> Result<ChildNode, XMLError> {
        for child in &self.children {
            if child.borrow().raw_tag == tag.as_bytes() {
                return Ok(child.clone())
            } else { 
                if let Ok(t) = child.clone().borrow().get_tag(tag) {
                    return Ok(t);
                };
            }
        }

        Err(XMLError::tag_not_present(tag))
    }
    
    /// Get attribute, coerces the return type
    /// Returns an Error if attribute doesn't exist or if type coercion fails
    pub fn get_attribute<T>(&self, attr: &str) -> Result<T, XMLError> 
    where T: FromStr + Default,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static // TODO: How do I make this less ugly?
    {
        match self.get_attribute_raw(attr) {
            Some(raw_attr) => match from_utf8(&raw_attr) {
                Ok(str) => Ok(convert::<T>(str)?),
                Err(_) => Err(XMLError::invalid("Attribute is not a valid UTF-8 string")),
            },
            None => Err(XMLError::attr_not_present(attr)),
        }
    }

    pub fn get_attribute_raw(&self, attr: &str) -> Option<&[u8]> {
        for attribute in &self.attributes {
            if attribute.0 == attr.as_bytes() {
                return Some(&attribute.1)
            }
        }
        None
    }
}

///  Build an xml tree from mmp
///  We can then use the constructed tree to validate it.
pub fn build_tree<R>(file: R) -> Result<Node, XMLError>
where 
    R: BufRead + Seek
{
    let mut root_tree: Option<ChildNode> = None;
    let mut parent_stack: Vec<ChildNode> = Vec::new();
    let mut reader = Reader::from_reader(file);
    let mut buf: Vec<u8> = Vec::new();

    loop {
        use Event::*;
        match reader.read_event_into(&mut buf) {
            Ok(event) => match event {
                Start(ref e) | Empty(ref e) => {
                    let attributes = e
                        .attributes()
                        .filter_map(|f| f.ok())
                        .map(|f| (f.key.0.to_owned(), f.value.to_vec()))
                        .collect();
                    
                    let raw_tag = e.name().0.to_owned();

                    let node = Rc::new(RefCell::new(Node{ 
                        attributes,
                        raw_tag,
                        ..Default::default()
                    }));

                    // Make the parent tree this node if not set.
                    if root_tree.is_none() {
                        root_tree = Some(node.clone());
                    }

                    // If there's a parent node on the stack:
                    // 1) Add this node to the parent's children
                    // 2) Set this node's parent to this parent
                    //
                    // I totally remembered that stacks are LIFO and didn't use .first()
                    if let Some(parent) = parent_stack.last() { 
                        parent.borrow_mut().children.push(node.clone());
                        // Use Weak<T> to prevent reference cycles, which can cause memory leaks
                        node.borrow_mut().parent = Some(Rc::downgrade(&parent)) 
                    }

                    // Only push the node to stack if the event is not an empty element tag
                    // Took a while to figure out why "lmms-project" only has 1 child.
                    if let Start(_) = event {
                        parent_stack.push(node.clone());    
                    }         
                },

                End(_) => if !parent_stack.is_empty() { parent_stack.pop(); },

                // Comment(_) => todo!(),
                // CData(e) => {dbg!(e);}, // TODO: Needed for project notes
                // Decl(e) => {dbg!(e);},
                // PI(_) => todo!(),
                // DocType(e) => {dbg!(e);},
                Eof => break,
                _ => continue,
            },
            Err(e) => return Err(XMLError::ParseError(e)),
        }
        buf.clear();
    }

    let Some(xml) = root_tree else {
        return Err(XMLError::invalid("Expected an XML tree but it does not exist"));
    };

    Ok(xml.take())
}