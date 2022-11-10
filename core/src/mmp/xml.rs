use quick_xml::events::attributes::{Attributes, Attribute};
use quick_xml::{self, Reader, events::Event};
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, Seek, BufReader};
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::str::{from_utf8, FromStr};

type ParentNode = Weak<RefCell<Node>>;
type ChildNode= Rc<RefCell<Node>>;

// TODO: Use thiserror
#[derive(Debug)]
pub enum XMLError {
    Invalid,
    AttrNotPresent,
    TagNotPresent,
    // FailedCoersion(DynError),
}

type DynError = dyn std::error::Error + Send + Sync + 'static;

/// Helper function to convert strings to different types
fn convert<T>(str: &str) -> T
where
    T: FromStr + Default, 
    // <T as FromStr>::Err: std::error::Error + Send + Sync + 'static
{
    T::from_str(str).unwrap_or_default()
}

#[derive(Default, Debug)]
pub struct Node {
    parent: Option<ParentNode>,
    children: Vec<ChildNode>,
    raw_tag: Vec<u8>,
    attributes: Vec<(Vec<u8>, Vec<u8>)>
}

impl Node {
    // Current tag of element
    pub fn tag(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.raw_tag)
    }

    pub fn attributes(&self) -> () {
        todo!()
    }

    // Recursively traverse tree to find a desired tag
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

        Err(XMLError::TagNotPresent)
    }
    
    // Get attribute, coerces the return type
    // Returns an Error if attribute doesn't exist or if type coersion fails
    pub fn get_attribute<T>(&self, attr: &str) -> Result<T, XMLError> 
    where T: FromStr + Default
    {
        match self.get_attribute_raw(attr) {
            Some(raw_attr) => match from_utf8(&raw_attr) {
                Ok(str) => Ok(convert::<T>(str)), // TODO
                Err(_) => Err(XMLError::AttrNotPresent),
            },
            None => Err(XMLError::AttrNotPresent),
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

//  Build an xml tree from mmp
//  We can then use the constructed tree to validate it.
pub fn build_tree<R>(file: R) -> ChildNode
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
                    // 2) Make this node's parent to this parent
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
                // Empty(_) => todo!(),
                // Text(_) => todo!(),
                // Comment(_) => todo!(),
                CData(e) => {dbg!(e);}, // TODO: Needed for project notes
                Decl(e) => {dbg!(e);},
                // PI(_) => todo!(),
                DocType(e) => {dbg!(e);},
                Eof => break,
                _ => continue,
            },
            Err(e) => break, // TODO: replace with error
        }
        buf.clear();
    }
    root_tree.unwrap()
}


#[test] 
fn a() {
    let a = File::open("../format.mmp").unwrap();
    let xml = build_tree(BufReader::new(a));
    let root = xml.borrow(); // The root of the lmms project

    assert_eq!(root.tag(), "lmms-project");
    dbg!(root.tag());
    dbg!(root.children.len());

    // let a = &xml.get_tag("trackcontainer").unwrap();
    // head.borrow().attributes
    match (root.get_tag("head") , root.get_tag("song")) {
        (Ok(_), Ok(_)) => {println!("Valid!")},
        _ => {println!("INVALID")}
    }
    let version: _ = root.get_attribute::<usize>("version");
    let song_type: _ = root.get_attribute::<String>("type");
    let creator: _ = root.get_attribute::<String>("creator");
    let creatorversion:_ = root.get_attribute::<String>("creatorversion"); 

    let song_info = root.get_tag("song").unwrap();
    let song_info = song_info.borrow();

    let head = root.get_tag("head").unwrap();
    let head = head.borrow();

    // dbg!(song_info.children.len());
    for e in &song_info.children {
        println!("{}", e.borrow().tag())
    }
    for attr in root.attributes.iter().map(|(k,v)| (String::from_utf8_lossy(k),String::from_utf8_lossy(v))) {
        dbg!(attr);
    };

    // dbg!(version);
    // dbg!(song_type);
    // dbg!(creator);
    // dbg!(creatorversion);

    
}