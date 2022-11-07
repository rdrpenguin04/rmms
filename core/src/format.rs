use std::io::{self, BufRead, Seek};
use std::str::FromStr;

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

#[derive(Debug)]
pub struct ProjectFile {
    head: ProjectHead,
    tracks: Vec<Track>,
}

impl ProjectFile {
    pub fn empty() -> Self {
        ProjectFile {
            head: ProjectHead {
                bpm: 140.0,
                vol: 100.0,
                time_sig: (4, 4),
                master_pitch: 0,
            },
            tracks: Vec::new(),
        }
    }
}

impl ProjectFile {
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

    pub fn load_xml(file: impl BufRead + Seek) -> io::Result<Self> {
        use quick_xml::{events::Event, Error, Reader};

        let mut result = ProjectFile::empty();
        let mut reader = Reader::from_reader(file);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"lmms-project" => loop {
                        match reader.read_event_into(&mut buf) {
                            Ok(Event::Empty(e)) => match e.name().as_ref() {
                                b"head" => {
                                    for attrib in e.attributes() {
                                        let Ok(attrib) = attrib else { return Err(io::Error::new(io::ErrorKind::InvalidData, "bad attribute")) };
                                        match attrib.key.as_ref() {
                                            b"bpm" => {
                                                result.head.bpm = f32::from_str(
                                                    &attrib.unescape_value().map_err(|x| {
                                                        io::Error::new(
                                                            io::ErrorKind::InvalidData,
                                                            x,
                                                        )
                                                    })?,
                                                )
                                                .map_err(|x| {
                                                    io::Error::new(io::ErrorKind::InvalidData, x)
                                                })?;
                                            }
                                            b"mastervol" => {
                                                result.head.vol = f32::from_str(
                                                    &attrib.unescape_value().map_err(|x| {
                                                        io::Error::new(
                                                            io::ErrorKind::InvalidData,
                                                            x,
                                                        )
                                                    })?,
                                                )
                                                .map_err(|x| {
                                                    io::Error::new(io::ErrorKind::InvalidData, x)
                                                })?;
                                            }
                                            b"timesig_denominator" => {
                                                result.head.time_sig.1 = u8::from_str(
                                                    &attrib.unescape_value().map_err(|x| {
                                                        io::Error::new(
                                                            io::ErrorKind::InvalidData,
                                                            x,
                                                        )
                                                    })?,
                                                )
                                                .map_err(|x| {
                                                    io::Error::new(io::ErrorKind::InvalidData, x)
                                                })?;
                                            }
                                            b"timesig_numerator" => {
                                                result.head.time_sig.0 = u8::from_str(
                                                    &attrib.unescape_value().map_err(|x| {
                                                        io::Error::new(
                                                            io::ErrorKind::InvalidData,
                                                            x,
                                                        )
                                                    })?,
                                                )
                                                .map_err(|x| {
                                                    io::Error::new(io::ErrorKind::InvalidData, x)
                                                })?;
                                            }
                                            b"masterpitch" => {
                                                result.head.master_pitch = i8::from_str(
                                                    &attrib.unescape_value().map_err(|x| {
                                                        io::Error::new(
                                                            io::ErrorKind::InvalidData,
                                                            x,
                                                        )
                                                    })?,
                                                )
                                                .map_err(|x| {
                                                    io::Error::new(io::ErrorKind::InvalidData, x)
                                                })?;
                                            }
                                            x => todo!("{}", std::str::from_utf8(x).unwrap()),
                                        }
                                    }
                                }
                                x => todo!("{}", std::str::from_utf8(x).unwrap()),
                            },
                            Ok(Event::Text(e)) => {
                                for c in e.as_ref() {
                                    if !char::from(*c).is_whitespace() {
                                        todo!()
                                    }
                                }
                            }
                            Ok(Event::End(e)) => match e.name().as_ref() {
                                b"lmms-project" => break,
                                _ => {
                                    return Err(io::Error::new(
                                        io::ErrorKind::InvalidData,
                                        "unexpected end tag",
                                    ))
                                }
                            },
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
                Ok(Event::Text(e)) => {
                    for c in e.as_ref() {
                        if !char::from(*c).is_whitespace() {
                            todo!()
                        }
                    }
                }
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
