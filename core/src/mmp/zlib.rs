use std::io::{Seek, BufRead, BufReader, SeekFrom};
use crate::mmp::xml::{self, Node, XMLError};
use flate2::read::ZlibDecoder;

pub fn decompress<R>(mut file: R) -> Result<Node, XMLError>
where R: BufRead + Seek 
{
    let _ = file.seek(SeekFrom::Current(4)); // Skip 4 bytes in compressed LMMS data (mmpz/xptz) 
    let zlib = ZlibDecoder::new(file);

    xml::build_tree(BufReader::new(zlib))
}
