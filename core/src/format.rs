use std::io::{self, BufRead, Seek};
use std::str::FromStr;

use quick_xml::{
    events::{BytesStart, Event},
    Error, Reader,
};

#[derive(Debug)]
pub struct InstrumentTrack {}

#[derive(Debug)]
pub enum TrackBody {
    Instrument(InstrumentTrack),
    Pattern, // Beat & Bassline. TODO: Deprecate and convert to something else
    Sample(),
    Unused3, // Formerly EventTrack.
    Unused4, // Formerly VideoTrack.
    Automation(),
    HiddenAutomation(), // Global automation track. TODO: Un-hide
}

#[allow(dead_code)] // TODO: Don't have dead code
#[derive(Debug)]
pub struct Track {
    name: String,
    mute: bool,
    solo: bool,
    body: TrackBody,
}

#[derive(Debug)]
pub struct ProjectHead {
    bpm: f32,
    vol: f32,
    time_sig: (u8, u8),
    master_pitch: i8,
}

impl Default for ProjectHead {
    fn default() -> Self {
        Self {
            bpm: 140.0,
            vol: 100.0,
            time_sig: (4, 4),
            master_pitch: 0,
        }
    }
}

#[derive(Debug)]
pub struct ProjectFile {
    head: ProjectHead,
    tracks: Vec<Track>,
}

impl ProjectFile {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            head: ProjectHead::default(),
            tracks: Vec::new(),
        }
    }
}

macro_rules! assert_whitespace {
    ($x:expr) => {
        for c in $x.as_ref() {
            if !char::from(*c).is_whitespace() {
                todo!()
            }
        }
    };
}

impl ProjectFile {
    /// # Errors
    /// This method returns an error if the underlying stream breaks or if the file is invalid
    ///
    /// # Panics
    /// This method panics if the file is unsupported. TODO: Don't panic.
    pub fn load(mut file: impl BufRead + Seek) -> io::Result<Self> {
        let mut first_two = [0; 2];
        file.read_exact(&mut first_two)?;
        file.rewind()?;
        if first_two == [b'<', b'?'] {
            // MMP file
            Self::load_xml(file)
        } else if matches!(first_two[0], b'{' | b'[') {
            todo!("New project format probably");
        } else {
            todo!("MMPZ");
        }
    }

    fn load_head(e: &BytesStart) -> io::Result<ProjectHead> {
        let mut head = ProjectHead::default();
        for attrib in e.attributes() {
            let Ok(attrib) = attrib else { return Err(io::Error::new(io::ErrorKind::InvalidData, "bad attribute")) };
            match attrib.key.as_ref() {
                b"bpm" => {
                    head.bpm = f32::from_str(
                        &attrib
                            .unescape_value()
                            .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?,
                    )
                    .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?;
                }
                b"mastervol" => {
                    head.vol = f32::from_str(
                        &attrib
                            .unescape_value()
                            .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?,
                    )
                    .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?;
                }
                b"timesig_denominator" => {
                    head.time_sig.1 = u8::from_str(
                        &attrib
                            .unescape_value()
                            .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?,
                    )
                    .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?;
                }
                b"timesig_numerator" => {
                    head.time_sig.0 = u8::from_str(
                        &attrib
                            .unescape_value()
                            .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?,
                    )
                    .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?;
                }
                b"masterpitch" => {
                    head.master_pitch = i8::from_str(
                        &attrib
                            .unescape_value()
                            .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?,
                    )
                    .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?;
                }
                x => todo!("{}", std::str::from_utf8(x).unwrap()),
            }
        }
        Ok(head)
    }

    #[allow(clippy::needless_pass_by_value)] // TODO: Re-evaluate
    fn load_track(reader: &mut Reader<impl BufRead + Seek>, _e: BytesStart) -> io::Result<Track> {
        let mut buf = Vec::new();

        let track = Track {
            name: String::new(),
            solo: false,
            mute: false,
            body: TrackBody::Unused3, // Temp
        };
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"track" => break,
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "unexpected end tag",
                        ))
                    }
                },
                Ok(Event::Text(e)) => assert_whitespace!(e),
                Ok(Event::Eof) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "unexpected EOF",
                    ))
                }
                Ok(x) => todo!("{:?}", x),
                Err(Error::Io(x)) => return Err(x),
                Err(x) => return Err(io::Error::new(io::ErrorKind::InvalidData, x)),
            }
        }
        Ok(track)
    }

    /// # Errors
    /// This method returns an error if the underlying stream breaks or if the file is invalid
    #[allow(clippy::missing_panics_doc, clippy::too_many_lines)] // TODO: improve this
    pub fn load_xml(file: impl BufRead + Seek) -> io::Result<Self> {
        let mut result = Self::empty();
        let mut reader = Reader::from_reader(file);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"lmms-project" => loop {
                        match reader.read_event_into(&mut buf) {
                            Ok(Event::Empty(e)) => match e.name().as_ref() {
                                b"head" => {
                                    result.head = Self::load_head(&e)?;
                                }
                                x => todo!("{}", std::str::from_utf8(x).unwrap()),
                            },
                            Ok(Event::Start(e)) => match e.name().as_ref() {
                                b"song" => loop {
                                    match reader.read_event_into(&mut buf) {
                                        Ok(Event::Start(e)) => match e.name().as_ref() {
                                            b"trackcontainer" => loop {
                                                match reader.read_event_into(&mut buf) {
                                                    Ok(Event::Start(e)) => {
                                                        match e.name().as_ref() {
                                                            b"track" => result.tracks.push(
                                                                Self::load_track(&mut reader, e)?,
                                                            ),
                                                            x => todo!(
                                                                "{}",
                                                                std::str::from_utf8(x).unwrap()
                                                            ),
                                                        }
                                                    }
                                                    Ok(Event::End(e)) => match e.name().as_ref() {
                                                        b"trackcontainer" => break,
                                                        _ => {
                                                            return Err(io::Error::new(
                                                                io::ErrorKind::InvalidData,
                                                                "unexpected end tag",
                                                            ))
                                                        }
                                                    },
                                                    Ok(Event::Text(e)) => assert_whitespace!(e),
                                                    Ok(Event::Eof) => {
                                                        return Err(io::Error::new(
                                                            io::ErrorKind::UnexpectedEof,
                                                            "unexpected EOF",
                                                        ))
                                                    }
                                                    Ok(x) => todo!("{:?}", x),
                                                    Err(Error::Io(x)) => return Err(x),
                                                    Err(x) => {
                                                        return Err(io::Error::new(
                                                            io::ErrorKind::InvalidData,
                                                            x,
                                                        ))
                                                    }
                                                }
                                            },
                                            x => todo!("{}", std::str::from_utf8(x).unwrap()),
                                        },
                                        Ok(Event::End(e)) => match e.name().as_ref() {
                                            b"song" => break,
                                            _ => {
                                                return Err(io::Error::new(
                                                    io::ErrorKind::InvalidData,
                                                    "unexpected end tag",
                                                ))
                                            }
                                        },
                                        Ok(Event::Text(e)) => assert_whitespace!(e),
                                        Ok(Event::Eof) => {
                                            return Err(io::Error::new(
                                                io::ErrorKind::UnexpectedEof,
                                                "unexpected EOF",
                                            ))
                                        }
                                        Ok(x) => todo!("{:?}", x),
                                        Err(Error::Io(x)) => return Err(x),
                                        Err(x) => {
                                            return Err(io::Error::new(
                                                io::ErrorKind::InvalidData,
                                                x,
                                            ))
                                        }
                                    }
                                },
                                x => todo!("{}", std::str::from_utf8(x).unwrap()),
                            },
                            Ok(Event::End(e)) => match e.name().as_ref() {
                                b"lmms-project" => break,
                                _ => {
                                    return Err(io::Error::new(
                                        io::ErrorKind::InvalidData,
                                        "unexpected end tag",
                                    ))
                                }
                            },
                            Ok(Event::Text(e)) => assert_whitespace!(e),
                            Ok(Event::Eof) => {
                                return Err(io::Error::new(
                                    io::ErrorKind::UnexpectedEof,
                                    "unexpected EOF",
                                ))
                            }
                            Ok(x) => todo!("{:?}", x),
                            Err(Error::Io(x)) => return Err(x),
                            Err(x) => return Err(io::Error::new(io::ErrorKind::InvalidData, x)),
                        }
                    },
                    _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected tag")),
                },
                Ok(Event::Text(e)) => assert_whitespace!(e),
                Ok(Event::Comment(_) | Event::Decl(_) | Event::DocType(_) | Event::CData(_)) => {}
                Ok(Event::PI(_)) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected PI"))
                }
                Ok(Event::Eof) => break,
                Ok(x) => todo!("{:?}", x),
                Err(Error::Io(x)) => return Err(x),
                Err(x) => return Err(io::Error::new(io::ErrorKind::InvalidData, x)),
            }
        }

        Ok(result)
    }
}
