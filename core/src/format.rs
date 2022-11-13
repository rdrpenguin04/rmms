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
    Instrument(Option<InstrumentTrack>),
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
                Err(invalid_data!("unexpected stray text"))?
            }
        }
    };
}

macro_rules! invalid_data {
    ($x:expr) => {
        io::Error::new(io::ErrorKind::InvalidData, $x)
    };
}

macro_rules! unexpected_eof {
    () => {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "unexpected EOF",
        ))?
    };
}

/// Helper function to convert strings to different types
fn convert<T>(str: &str) -> io::Result<T>
where
    T: FromStr + Default,
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    T::from_str(str).map_err(|x| invalid_data!(x))
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

    fn load_head(e: &BytesStart, is_empty: bool) -> io::Result<ProjectHead> {
        let mut head = ProjectHead::default();

        for attr in e.attributes() {
            let Ok(attr) = attr else { Err(invalid_data!("bad attribute"))? };
            let value = attr.unescape_value().map_err(|x| invalid_data!(x))?;
            match attr.key.as_ref() {
                b"bpm" => head.bpm = convert(&value)?,
                b"mastervol" => head.vol = convert(&value)?,
                b"masterpitch" => head.master_pitch = convert(&value)?,
                b"timesig_numerator" => head.time_sig.0 = convert(&value)?,
                b"timesig_denominator" => head.time_sig.1 = convert(&value)?,
                x => todo!("{}", std::str::from_utf8(x).unwrap()),
            }
        }
        if !is_empty {
            todo!(); // What does this mean here?
        }
        Ok(head)
    }

    fn load_instrument_track(
        _reader: &mut Reader<impl BufRead + Seek>,
        _e: &BytesStart,
        _is_empty: bool,
    ) -> io::Result<InstrumentTrack> {
        todo!()
    }

    fn load_track(
        reader: &mut Reader<impl BufRead + Seek>,
        e: &BytesStart,
        is_empty: bool,
    ) -> io::Result<Track> {
        let mut buf = Vec::new();

        let mut track = Track {
            name: String::new(),
            solo: false,
            mute: false,
            body: TrackBody::Instrument(None), // Temp
        };

        for attr in e.attributes() {
            let Ok(attr) = attr else { Err(invalid_data!("bad attribute"))? };
            let value = attr.unescape_value().map_err(|x| invalid_data!(x))?;
            match attr.key.as_ref() {
                b"name" => track.name = value.into(),
                b"solo" => track.solo = value == "1",
                b"muted" => track.mute = value == "1",
                b"type" => {
                    track.body = match value.as_ref() {
                        "0" => TrackBody::Instrument(None),
                        "1" => TrackBody::Pattern,
                        "2" => TrackBody::Sample(),
                        "3" => TrackBody::Unused3,
                        "4" => TrackBody::Unused4,
                        "5" => TrackBody::Automation(),
                        "6" => TrackBody::HiddenAutomation(),
                        x => todo!("{x}"),
                    }
                }
                b"mutedBeforeSolo" => {} // GUI element
                x => todo!("{}", std::str::from_utf8(x).unwrap()),
            }
        }

        if !is_empty {
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(e)) => match e.name().as_ref() {
                        b"instrumenttrack" => match &mut track.body {
                            TrackBody::Instrument(x) => {
                                *x = Some(Self::load_instrument_track(reader, &e, false)?);
                            }
                            _ => Err(invalid_data!("instrumenttrack tag in non-instrument track"))?,
                        },
                        x => todo!("{}", std::str::from_utf8(x).unwrap()),
                    },
                    Ok(Event::End(e)) => match e.name().as_ref() {
                        b"track" => break,
                        _ => Err(invalid_data!("unexpected end tag"))?,
                    },
                    Ok(Event::Text(e)) => assert_whitespace!(e),
                    Ok(Event::Eof) => unexpected_eof!(),
                    Ok(x) => todo!("{:?}", x),
                    Err(Error::Io(x)) => return Err(x),
                    Err(x) => return Err(io::Error::new(io::ErrorKind::InvalidData, x)),
                }
            }
        }
        Ok(track)
    }

    /// # Errors
    /// This method returns an error if the underlying stream breaks or if the file is invalid
    #[allow(clippy::missing_panics_doc)] // TODO: don't panic
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
                                    result.head = Self::load_head(&e, true)?;
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
                                                                Self::load_track(
                                                                    &mut reader,
                                                                    &e,
                                                                    false,
                                                                )?,
                                                            ),
                                                            x => todo!(
                                                                "{}",
                                                                std::str::from_utf8(x).unwrap()
                                                            ),
                                                        }
                                                    }
                                                    Ok(Event::End(e)) => match e.name().as_ref() {
                                                        b"trackcontainer" => break,
                                                        _ => Err(invalid_data!(
                                                            "unexpected end tag"
                                                        ))?,
                                                    },
                                                    Ok(Event::Text(e)) => assert_whitespace!(e),
                                                    Ok(Event::Eof) => unexpected_eof!(),
                                                    Ok(x) => todo!("{:?}", x),
                                                    Err(Error::Io(x)) => return Err(x),
                                                    Err(x) => Err(invalid_data!(x))?,
                                                }
                                            },
                                            x => todo!("{}", std::str::from_utf8(x).unwrap()),
                                        },
                                        Ok(Event::End(e)) => match e.name().as_ref() {
                                            b"song" => break,
                                            _ => Err(invalid_data!("unexpected end tag"))?,
                                        },
                                        Ok(Event::Text(e)) => assert_whitespace!(e),
                                        Ok(Event::Eof) => unexpected_eof!(),
                                        Ok(x) => todo!("{:?}", x),
                                        Err(Error::Io(x)) => return Err(x),
                                        Err(x) => Err(invalid_data!(x))?,
                                    }
                                },
                                x => todo!("{}", std::str::from_utf8(x).unwrap()),
                            },
                            Ok(Event::End(e)) => match e.name().as_ref() {
                                b"lmms-project" => break,
                                _ => Err(invalid_data!("unexpected end tag"))?,
                            },
                            Ok(Event::Text(e)) => assert_whitespace!(e),
                            Ok(Event::Eof) => unexpected_eof!(),
                            Ok(x) => todo!("{:?}", x),
                            Err(Error::Io(x)) => return Err(x),
                            Err(x) => Err(invalid_data!(x))?,
                        }
                    },
                    _ => Err(invalid_data!("unexpected tag"))?,
                },
                Ok(Event::Text(e)) => assert_whitespace!(e),
                Ok(Event::Comment(_) | Event::Decl(_) | Event::DocType(_) | Event::CData(_)) => {}
                Ok(Event::PI(_)) => Err(invalid_data!("unexpected PI"))?,
                Ok(Event::Eof) => break,
                Ok(x) => todo!("{:?}", x),
                Err(Error::Io(x)) => return Err(x),
                Err(x) => Err(invalid_data!(x))?,
            }
        }

        Ok(result)
    }
}
