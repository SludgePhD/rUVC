use std::{fmt, time::Duration};

use bitflags::bitflags;
use zerocopy::{AsBytes, FromBytes};

/// Raw value of entity controls.

pub trait ControlValue {
    type Buf: Default + AsMut<[u8]>;

    fn decode(buf: &[u8]) -> Self;
    fn encode(&self, buf: &mut [u8]);
}

impl ControlValue for bool {
    type Buf = [u8; 1];

    fn decode(buf: &[u8]) -> Self {
        match buf[0] {
            0 => false,
            1 => true,
            n => {
                log::warn!("invalid bool value (should be 0 or 1 only): {}", n);
                true
            }
        }
    }

    fn encode(&self, buf: &mut [u8]) {
        buf[0] = *self as u8;
    }
}

impl ControlValue for u8 {
    type Buf = [u8; 1];

    fn decode(buf: &[u8]) -> Self {
        buf[0]
    }

    fn encode(&self, buf: &mut [u8]) {
        buf[0] = *self;
    }
}

impl ControlValue for i8 {
    type Buf = [u8; 1];

    fn decode(buf: &[u8]) -> Self {
        buf[0] as i8
    }

    fn encode(&self, buf: &mut [u8]) {
        buf[0] = *self as u8;
    }
}

impl ControlValue for u16 {
    type Buf = [u8; 2];

    fn decode(buf: &[u8]) -> Self {
        let mut bytes = [0; 2];
        bytes.copy_from_slice(buf);
        Self::from_le_bytes(bytes)
    }

    fn encode(&self, buf: &mut [u8]) {
        buf.copy_from_slice(&self.to_le_bytes())
    }
}

impl ControlValue for i16 {
    type Buf = [u8; 2];

    fn decode(buf: &[u8]) -> Self {
        let mut bytes = [0; 2];
        bytes.copy_from_slice(buf);
        Self::from_le_bytes(bytes)
    }

    fn encode(&self, buf: &mut [u8]) {
        buf.copy_from_slice(&self.to_le_bytes())
    }
}

impl ControlValue for u32 {
    type Buf = [u8; 4];

    fn decode(buf: &[u8]) -> Self {
        let mut bytes = [0; 4];
        bytes.copy_from_slice(buf);
        Self::from_le_bytes(bytes)
    }

    fn encode(&self, buf: &mut [u8]) {
        buf.copy_from_slice(&self.to_le_bytes())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PowerLineFrequency {
    Disabled = 0,
    Freq50Hz = 1,
    Freq60Hz = 2,
    Auto = 3,
}

impl ControlValue for PowerLineFrequency {
    type Buf = [u8; 1];

    fn decode(buf: &[u8]) -> Self {
        match buf[0] {
            0 => Self::Disabled,
            1 => Self::Freq50Hz,
            2 => Self::Freq60Hz,
            3 => Self::Auto,
            n => {
                log::warn!("invalid power line frequency value {}", n);
                Self::Disabled
            }
        }
    }

    fn encode(&self, buf: &mut [u8]) {
        buf[0] = (*self) as u8;
    }
}

#[derive(Debug)]
pub struct WhiteBalanceComponents {
    blue: u16,
    red: u16,
}

impl WhiteBalanceComponents {
    pub fn new(blue: u16, red: u16) -> Self {
        Self { blue, red }
    }
}

impl ControlValue for WhiteBalanceComponents {
    type Buf = [u8; 4];

    fn decode(buf: &[u8]) -> Self {
        let mut blue = [0; 2];
        let mut red = [0; 2];
        blue.copy_from_slice(&buf[0..2]);
        red.copy_from_slice(&buf[2..4]);
        Self {
            blue: u16::from_le_bytes(blue),
            red: u16::from_le_bytes(red),
        }
    }

    fn encode(&self, buf: &mut [u8]) {
        buf[0..2].copy_from_slice(&self.blue.to_le_bytes());
        buf[2..4].copy_from_slice(&self.red.to_le_bytes());
    }
}

bitflags! {
    pub struct AutoExposureMode: u8 {
        const MANUAL = 1 << 0;
        const AUTO = 1 << 1;
        const SHUTTER_PRIORITY = 1 << 2;
        const APERTURE_PRIORITY = 1 << 3;
    }
}

impl ControlValue for AutoExposureMode {
    type Buf = [u8; 1];

    fn decode(buf: &[u8]) -> Self {
        Self::from_bits_truncate(buf[0])
    }

    fn encode(&self, buf: &mut [u8]) {
        buf[0] = self.bits();
    }
}

#[derive(Clone, Copy)]
pub struct ExposureTimeAbs(u32);

impl ExposureTimeAbs {
    /// Rounds and clamps a duration to fit the available range.
    pub fn from_duration(dur: Duration) -> Self {
        // Exposure time is in units of 0.0001 seconds, or 100µs.
        let units = dur.as_micros() / 100;
        let clamped = units.clamp(1, u32::MAX.into());
        Self(clamped as u32)
    }

    pub fn as_duration(&self) -> Duration {
        Duration::from_micros(u64::from(self.0) * 100)
    }
}

impl fmt::Debug for ExposureTimeAbs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_duration().fmt(f)
    }
}

impl ControlValue for ExposureTimeAbs {
    type Buf = [u8; 4];

    fn decode(buf: &[u8]) -> Self {
        let mut bytes = [0; 4];
        bytes.copy_from_slice(buf);
        Self(u32::from_le_bytes(bytes))
    }

    fn encode(&self, buf: &mut [u8]) {
        buf.copy_from_slice(&self.0.to_le_bytes());
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FocusRel {
    focus_rel: i8,
    speed: u8,
}

impl FocusRel {
    pub fn new(focus_rel: i8, speed: u8) -> Self {
        Self { focus_rel, speed }
    }
}

impl ControlValue for FocusRel {
    type Buf = [u8; 2];

    fn decode(buf: &[u8]) -> Self {
        Self {
            focus_rel: buf[0] as i8,
            speed: buf[1],
        }
    }

    fn encode(&self, buf: &mut [u8]) {
        buf[0] = self.focus_rel as u8;
        buf[1] = self.speed;
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum FocusSimple {
    FullRange = 0x00,
    Macro = 0x01,
    People = 0x02,
    Scene = 0x03,
}

impl ControlValue for FocusSimple {
    type Buf = [u8; 1];

    fn decode(buf: &[u8]) -> Self {
        match buf[0] {
            0x00 => Self::FullRange,
            0x01 => Self::Macro,
            0x02 => Self::People,
            0x03 => Self::Scene,
            n => {
                log::warn!("invalid simple focus value {}", n);
                Self::FullRange
            }
        }
    }

    fn encode(&self, buf: &mut [u8]) {
        buf[0] = *self as u8;
    }
}

#[derive(Default, AsBytes, FromBytes, Debug, Clone, Copy)]
#[repr(C, packed)]
#[allow(non_snake_case)]
pub struct ProbeCommitControls {
    pub bmHint: ProbeHint,
    pub bFormatIndex: u8,
    pub bFrameIndex: u8,
    pub dwFrameInterval: u32,
    pub wKeyFrameRate: u16,
    pub wPFrameRate: u16,
    pub wCompQuality: u16,
    pub wCompWindowSize: u16,
    pub wDelay: u16,
    pub dwMaxVideoFrameSize: u32,
    pub dwMaxPayloadTransferSize: u32,
    // FIXME: there's a MAJOR bug in the Leap Motion firmware that will make the device fail when
    // the fields below are included, presumably because it cannot handle more data than it expects.
    // The effect is that `GET_CUR(PROBE)` returns a 0 value in `dwFrameInterval` instead of the
    // value sent by the preceding `SET_CUR(PROBE)`.
    /*pub dwClockFrequency: u32,
    pub bmFramingInfo: u8,
    pub bPreferedVersion: u8, // (sic)
    pub bMinVersion: u8,
    pub bMaxVersion: u8,
    pub bUsage: u8,
    pub bBitDepthLuma: u8,
    pub bmSettings: u8,
    pub bMaxNumberOfRefFramesPlus1: u8,
    pub bmRateControlModes: u16,
    pub bmLayoutPerStream: u64,*/
}

impl ControlValue for ProbeCommitControls {
    type Buf = ProbeCommitControlsBuf;

    fn decode(buf: &[u8]) -> Self {
        Self::read_from(buf).expect("couldn't decode `ProbeCommitControls`")
    }

    fn encode(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self.as_bytes());
    }
}

// FIXME no `Default` impl for large arrays
#[derive(Clone, Copy, Debug)]
pub struct ProbeCommitControlsBuf([u8; std::mem::size_of::<ProbeCommitControls>()]);

impl Default for ProbeCommitControlsBuf {
    fn default() -> Self {
        Self([0; std::mem::size_of::<ProbeCommitControls>()])
    }
}

impl AsMut<[u8]> for ProbeCommitControlsBuf {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

bitflags! {
    #[derive(Default, AsBytes, FromBytes)]
    #[repr(transparent)]
    pub struct ProbeHint: u16 {
        const FIX_FRAME_INTERVAL = 1 << 0;
        const FIX_KEY_FRAME_RATE = 1 << 1;
        const FIX_P_FRAME_RATE = 1 << 2;
        const FIX_COMP_QUALITY = 1 << 3;
        const FIX_COMP_WINDOW_SIZE = 1 << 4;
    }
}
