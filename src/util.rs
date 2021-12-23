use std::{
    fmt,
    io::{self, Read},
    time::Duration,
};

use byteorder::{ReadBytesExt, LE};
use uuid::Uuid;

use crate::topo::{SourceId, TermId, UnitId};

/// primitive_enum! {}
macro_rules! primitive_enum {
    (
        $v:vis enum $name:ident: $native:ty {
            $(
                $( #[$variant_attrs:meta] )*
                $variant:ident = $value:expr
            ),+
            $(,)?
        }
    ) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        $v enum $name {
            $(
                $variant = $value,
            )+
        }

        impl $name {
            pub(crate) fn from_raw(raw: $native) -> Option<Self> {
                match raw {
                    $(
                        $value => Some(Self::$variant),
                    )+
                    _ => None,
                }
            }
        }

        #[allow(unreachable_patterns)]
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $(
                        Self::$variant => f.write_str(stringify!($variant)),
                    )+
                }
            }
        }
    };
}

pub(crate) fn split_descriptors(mut raw: &[u8]) -> impl Iterator<Item = (u8, &[u8])> {
    std::iter::from_fn(move || match raw {
        [length, descriptor_type, ..] => {
            let length = *length as usize;
            if length > raw.len() {
                log::warn!(
                    "descriptor length {} exceeds available data ({} bytes)",
                    length,
                    raw.len()
                );
                return None;
            }
            let (desc_data, next) = raw.split_at(length);

            raw = next;

            Some((*descriptor_type, desc_data))
        }
        [] => None,
        _ => {
            log::warn!("invalid trailing descriptor bytes: {:x?}", raw);
            None
        }
    })
}

pub(crate) trait BytesExt {
    fn read_length_prefixed_bitmask(&mut self) -> io::Result<u32>;
    fn read_bitmask(&mut self, len: u8) -> io::Result<u32>;
    fn read_nonzero_source_id(&mut self) -> io::Result<SourceId>;
    fn read_nonzero_term_id(&mut self) -> io::Result<TermId>;
    fn read_nonzero_unit_id(&mut self) -> io::Result<UnitId>;
    fn read_guid(&mut self) -> io::Result<Uuid>;
    fn read_time_100ns(&mut self) -> io::Result<Duration>;
}

impl BytesExt for &'_ [u8] {
    fn read_length_prefixed_bitmask(&mut self) -> io::Result<u32> {
        let len = self.read_u8()?;
        self.read_bitmask(len)
    }

    fn read_bitmask(&mut self, len: u8) -> io::Result<u32> {
        let len = usize::from(len);
        if len > self.len() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        if len > 4 {
            log::warn!("bitmask length {}, discarding data past 32 bits", len);
        }

        let (bytes, rest) = self.split_at(len);
        *self = rest;

        let mut buf = [0u8; 4];
        buf.iter_mut()
            .zip(bytes)
            .for_each(|(dest, src)| *dest = *src);
        Ok(u32::from_le_bytes(buf))
    }

    fn read_nonzero_source_id(&mut self) -> io::Result<SourceId> {
        SourceId::new(self.read_u8()?)
            .ok_or_else(|| io_err("bSourceID is 0, only non-zero numbers are allowed"))
    }

    fn read_nonzero_term_id(&mut self) -> io::Result<TermId> {
        TermId::new(self.read_u8()?)
            .ok_or_else(|| io_err("bTerminalID is 0, only non-zero numbers are allowed"))
    }

    fn read_nonzero_unit_id(&mut self) -> io::Result<UnitId> {
        UnitId::new(self.read_u8()?)
            .ok_or_else(|| io_err("bUnitID is 0, only non-zero numbers are allowed"))
    }

    fn read_guid(&mut self) -> io::Result<Uuid> {
        // Weird encoding, apparently the first 3 groups in a UUID are "numbers", the last 2 groups
        // are just "bytes", and USB-IF insists on encoding all numbers in little endian.
        let d1 = self.read_u32::<LE>()?;
        let d2 = self.read_u16::<LE>()?;
        let d3 = self.read_u16::<LE>()?;
        let mut d4 = [0; 8];
        self.read_exact(&mut d4)?;
        Ok(Uuid::from_fields(d1, d2, d3, &d4).unwrap())
    }

    fn read_time_100ns(&mut self) -> io::Result<Duration> {
        let units = self.read_u32::<LE>()?;
        Ok(Duration::from_nanos(u64::from(units) * 100))
    }
}

pub(crate) fn io_err_res<T, M>(msg: M) -> io::Result<T>
where
    M: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    Err(io_err(msg))
}

pub(crate) fn io_err<M>(msg: M) -> io::Error
where
    M: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::Other, msg)
}

#[derive(Clone, Copy)]
pub struct BcdVersion(pub(crate) u16);

impl fmt::Display for BcdVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let maj = self.0 >> 8;
        let min = self.0 & 0xff;
        write!(f, "{}.{}", maj, min)
    }
}

impl fmt::Debug for BcdVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
