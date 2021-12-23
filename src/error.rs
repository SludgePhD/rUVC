use std::{fmt, io};

pub struct Error {
    action: Option<Action>,
    kind: ErrorKind,
}

impl Error {
    pub(crate) fn with_action(kind: impl Into<ErrorKind>, action: Action) -> Self {
        Self {
            action: Some(action),
            kind: kind.into(),
        }
    }

    pub(crate) fn is_usb_timeout(&self) -> bool {
        matches!(&self.kind, ErrorKind::Rusb(rusb::Error::Timeout))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(action) = &self.action {
            write!(f, "error while {}: ", action)?;
        }

        match &self.kind {
            ErrorKind::Rusb(e) => write!(f, "{}", e),
            ErrorKind::Io(e) => write!(f, "{}", e),
            ErrorKind::Other(e) => write!(f, "{}", e),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Rusb(rusb::Error),
    Io(io::Error),
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ErrorKind {
    fn from(v: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Other(v)
    }
}

impl From<String> for ErrorKind {
    fn from(s: String) -> Self {
        Self::Other(s.into())
    }
}

impl From<&'_ str> for ErrorKind {
    fn from(s: &str) -> Self {
        Self::Other(s.into())
    }
}

impl From<rusb::Error> for ErrorKind {
    fn from(e: rusb::Error) -> Self {
        Self::Rusb(e)
    }
}

impl From<io::Error> for ErrorKind {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// A list of actions during which this library might encounter errors.
#[derive(Debug)]
pub(crate) enum Action {
    AccessingDeviceDescriptor,
    EnumeratingDevices,
    OpeningDevice,
    ReadingDeviceString,
    ReadingControl,
    WritingControl,
    StreamNegotiation,
    StreamRead,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Action::AccessingDeviceDescriptor => "accessing device descriptor",
            Action::EnumeratingDevices => "enumerating USB devices",
            Action::OpeningDevice => "opening UVC device",
            Action::ReadingDeviceString => "reading device strings",
            Action::ReadingControl => "reading a device control",
            Action::WritingControl => "writing a device control",
            Action::StreamNegotiation => "negotiating stream parameters",
            Action::StreamRead => "reading from the video stream",
        };
        f.write_str(s)
    }
}

pub(crate) trait ResultExt<T, E> {
    fn during(self, action: Action) -> Result<T, Error>;
}

impl<T, E: Into<ErrorKind>> ResultExt<T, E> for Result<T, E> {
    fn during(self, action: Action) -> Result<T, Error> {
        self.map_err(|e| Error::with_action(e, action))
    }
}

pub(crate) fn err<T>(err: impl Into<ErrorKind>, action: Action) -> Result<T, Error> {
    Err(Error::with_action(err, action))
}
