//! UVC device topology.
//!
//! UVC devices consist of:
//! - *Input Terminals*, which provide video data to the UVC device.
//! - *Output Terminals*, which transfer video data away from the UVC device.
//! - *Units*, which connect between other units or terminals, and process or reroute video data.

// TODO: wrap all bitflags structs in newtypes to hide the raw type

pub(crate) mod parse;

use std::{num::NonZeroU8, time::Duration};

use bitflags::bitflags;
use uuid::Uuid;

use crate::util::BcdVersion;

/// Identifies a video data source (either a [`Unit`], or an [`InputTerminal`]).
#[derive(Clone, Copy, Debug)]
pub struct SourceId(NonZeroU8);

impl SourceId {
    pub(crate) fn new(raw: u8) -> Option<Self> {
        NonZeroU8::new(raw).map(Self)
    }
}

/// Identifies an [`InputTerminal`] or an [`OutputTerminal`].
#[derive(Clone, Copy, Debug)]
pub struct TermId(NonZeroU8);

impl TermId {
    pub(crate) fn new(raw: u8) -> Option<Self> {
        NonZeroU8::new(raw).map(Self)
    }
}

/// Identifies an [`InputTerminal`] that is a [`CameraTerminal`].
#[derive(Clone, Copy, Debug)]
pub struct CameraId(TermId);

impl CameraId {
    pub(crate) fn as_raw(self) -> u8 {
        self.0 .0.into()
    }
}

impl From<CameraId> for TermId {
    fn from(id: CameraId) -> Self {
        id.0
    }
}

/// Identifies a [`Unit`].
#[derive(Clone, Copy, Debug)]
pub struct UnitId(NonZeroU8);

impl UnitId {
    pub(crate) fn new(raw: u8) -> Option<Self> {
        NonZeroU8::new(raw).map(Self)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProcessingUnitId(UnitId);

impl ProcessingUnitId {
    pub(crate) fn as_raw(self) -> u8 {
        self.0 .0.into()
    }
}

impl From<ProcessingUnitId> for UnitId {
    fn from(id: ProcessingUnitId) -> Self {
        id.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SelectorUnitId(UnitId);

impl From<SelectorUnitId> for UnitId {
    fn from(id: SelectorUnitId) -> Self {
        id.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExtensionUnitId(UnitId);

impl From<ExtensionUnitId> for UnitId {
    fn from(id: ExtensionUnitId) -> Self {
        id.0
    }
}

/// The device topology as reported by the Video Control interface descriptors.
#[derive(Debug)]
pub struct Topology {
    header: ControlHeader,
    units: Vec<UnitDesc>,
    inputs: Vec<InputTerminalDesc>,
    outputs: Vec<OutputTerminalDesc>,
}

impl Topology {
    pub fn camera_terminal_by_id(&self, id: CameraId) -> &CameraTerminalDesc {
        self.inputs
            .iter()
            .find(|inp| inp.as_camera_id().map_or(false, |cid| cid.0 .0 == id.0 .0))
            .map(|inp| inp.as_camera_desc().unwrap())
            .expect("could not find given `CameraId` in device topology")
    }

    pub fn processing_unit_by_id(&self, id: ProcessingUnitId) -> &ProcessingUnitDesc {
        self.units
            .iter()
            .filter_map(|unit| unit.as_processing_unit())
            .find(|unit| unit.id.0 .0 == id.0 .0)
            .expect("could not find processing unit in device topology")
    }

    pub fn units(&self) -> &[UnitDesc] {
        &self.units
    }

    pub fn inputs(&self) -> &[InputTerminalDesc] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[OutputTerminalDesc] {
        &self.outputs
    }
}

#[derive(Debug)]
pub struct ControlHeader {
    uvc_version: BcdVersion,
    total_len: u16,
    clock_freq_hz: u32,
    streaming_interfaces: Vec<u8>,
}

/// A unit declared by the Video Control Interface Descriptors.
#[derive(Debug)]
pub struct UnitDesc {
    kind: UnitKind,
}

impl UnitDesc {
    pub fn unit_kind(&self) -> &UnitKind {
        &self.kind
    }

    pub fn as_processing_unit(&self) -> Option<&ProcessingUnitDesc> {
        match &self.kind {
            UnitKind::Processing(unit) => Some(unit),
            _ => None,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum UnitKind {
    Selector(SelectorUnitDesc),
    Processing(ProcessingUnitDesc),
    Extension(ExtensionUnitDesc),
}

#[derive(Debug)]
pub struct SelectorUnitDesc {
    id: SelectorUnitId,
    inputs: Vec<SourceId>,
}

#[derive(Debug)]
pub struct ProcessingUnitDesc {
    id: ProcessingUnitId,
    source: SourceId,
    max_multiplier: u16,
    controls: ProcessingUnitControls,
    string: u8,
    standards: VideoStandards,
}

impl ProcessingUnitDesc {
    pub fn id(&self) -> ProcessingUnitId {
        self.id
    }

    pub fn controls(&self) -> ProcessingUnitControls {
        self.controls
    }
}

bitflags! {
    pub struct ProcessingUnitControls: u32 {
        const BRIGHTNESS                     = 1 << 0;
        const CONTRAST                       = 1 << 1;
        const HUE                            = 1 << 2;
        const SATURATION                     = 1 << 3;
        const SHARPNESS                      = 1 << 4;
        const GAMMA                          = 1 << 5;
        const WHITE_BALANCE_TEMPERATURE      = 1 << 6;
        const WHITE_BALANCE_COMPONENT        = 1 << 7;
        const BACKLIGHT_COMPENSATION         = 1 << 8;
        const GAIN                           = 1 << 9;
        const POWER_LINE_FREQUENCY           = 1 << 10;
        const HUE_AUTO                       = 1 << 11;
        const WHITE_BALANCE_TEMPERATURE_AUTO = 1 << 12;
        const WHITE_BALANCE_COMPONENT_AUTO   = 1 << 13;
        const DIGITAL_MULTIPLIER             = 1 << 14;
        const DIGITAL_MULTIPLIER_LIMIT       = 1 << 15;
        const ANALOG_VIDEO_STANDARD          = 1 << 16;
        const ANALOG_VIDEO_LOCK_STATUS       = 1 << 17;
        const CONTRAST_AUTO                  = 1 << 18;
    }
}

bitflags! {
    pub struct VideoStandards: u8 {
        const NONE         = 1 << 0;
        const NTSC_525_60  = 1 << 1;
        const PAL_625_50   = 1 << 2;
        const SECAM_625_50 = 1 << 3;
        const NTSC_625_50  = 1 << 4;
        const PAL_525_60   = 1 << 5;
    }
}

#[derive(Debug)]
pub struct ExtensionUnitDesc {
    id: ExtensionUnitId,
    extension_code: Uuid,
    num_controls: u8,
    inputs: Vec<SourceId>,
    controls_bitmap: Vec<u8>,
}

#[derive(Debug)]
pub struct OutputTerminalDesc {
    term_id: TermId,
    term_type: u16,
    assoc: Option<TermId>,
    source: SourceId,
    string: u8,
}

impl OutputTerminalDesc {
    pub fn terminal_type(&self) -> Option<OutputTerminalType> {
        OutputTerminalType::from_raw(self.term_type)
    }
}

#[derive(Debug)]
pub struct InputTerminalDesc {
    term_id: TermId,
    term_type: u16,
    assoc: Option<TermId>,
    string: u8,
    kind: InputTerminalKind,
}

impl InputTerminalDesc {
    pub fn terminal_type(&self) -> Option<InputTerminalType> {
        InputTerminalType::from_raw(self.term_type)
    }

    pub fn terminal_kind(&self) -> &InputTerminalKind {
        &self.kind
    }

    pub fn as_camera_id(&self) -> Option<CameraId> {
        match &self.kind {
            InputTerminalKind::Camera(_) => Some(CameraId(self.term_id)),
            _ => None,
        }
    }

    pub fn as_camera_desc(&self) -> Option<&CameraTerminalDesc> {
        match &self.kind {
            InputTerminalKind::Camera(cam) => Some(cam),
            _ => None,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum InputTerminalKind {
    Camera(CameraTerminalDesc),
    /// Misc. terminal without extra functionality (or with unimplemented functionality).
    Other,
}

#[derive(Debug)]
pub struct CameraTerminalDesc {
    objective_focal_length_min: u16,
    objective_focal_length_max: u16,
    ocular_focal_length: u16,
    controls: CameraControls,
}

impl CameraTerminalDesc {
    pub fn controls(&self) -> CameraControls {
        self.controls
    }
}

bitflags! {
    pub struct CameraControls: u32 {
        const SCANNING_MODE = 1 << 0;
        const AUTO_EXPOSURE_MODE = 1 << 1;
        const AUTO_EXPOSURE_PRIORITY = 1 << 2;
        const EXPOSURE_TIME_ABS = 1 << 3;
        const EXPOSURE_TIME_REL = 1 << 4;
        const FOCUS_ABS = 1 << 5;
        const FOCUS_REL = 1 << 6;
        const IRIS_ABS = 1 << 7;
        const IRIS_REL = 1 << 8;
        const ZOOM_ABS = 1 << 9;
        const ZOOM_REL = 1 << 10;
        const PAN_TILT_ABS = 1 << 11;
        const PAN_TILT_REL = 1 << 12;
        const ROLL_ABS = 1 << 13;
        const ROLL_REL = 1 << 14;

        const FOCUS_AUTO = 1 << 17;
        const PRIVACY = 1 << 18;
        const FOCUS_SIMPLE = 1 << 19;
        const WINDOW = 1 << 20;
        const REGION_OF_INTEREST = 1 << 21;
    }
}

primitive_enum! {
    pub enum InputTerminalType: u16 {
        UsbVendorSpecific = 0x0100,
        UsbStreaming = 0x0101,

        InVendorSpecific = 0x0200,
        InCamera = 0x0201,
        InMediaTransport = 0x0202,

        ExtVendorSpecific = 0x0400,
        ExtCompositeConnector = 0x0401,
        ExtSVideoConnector = 0x0402,
        ExtComponentConnector = 0x0403,
    }
}

primitive_enum! {
    pub enum OutputTerminalType: u16 {
        UsbVendorSpecific = 0x0100,
        UsbStreaming = 0x0101,

        OutVendorSpecific = 0x0300,
        OutDisplay = 0x0301,
        OutMediaTransport = 0x0302,

        ExtVendorSpecific = 0x0400,
        ExtCompositeConnector = 0x0401,
        ExtSVideoConnector = 0x0402,
        ExtComponentConnector = 0x0403,
    }
}

// TODO: put everything below here in its own file (and submodules for each payload type)

#[derive(Debug, Clone, Copy)]
pub struct StreamingInterfaceId(pub(crate) u8);

#[derive(Debug)]
pub struct StreamingInterfaceDesc {
    id: StreamingInterfaceId,
    kind: StreamingInterfaceKind,
    formats: Vec<Format>,
    frames: Vec<Frame>,
}

impl StreamingInterfaceDesc {
    pub fn id(&self) -> StreamingInterfaceId {
        self.id
    }

    pub fn formats(&self) -> &[Format] {
        &self.formats
    }

    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    pub fn frame_by_index(&self, index: FrameIndex) -> &Frame {
        self.frames.iter().find(|f| f.index().0 == index.0).unwrap()
    }

    pub fn endpoint_address(&self) -> u8 {
        match &self.kind {
            StreamingInterfaceKind::Input(k) => k.endpoint_address,
            StreamingInterfaceKind::Output(_) => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum StreamingInterfaceKind {
    Input(InputHeader),
    Output(OutputHeader),
}

#[derive(Debug)]
pub struct InputHeader {
    num_formats: u8,
    total_length: u16,
    endpoint_address: u8,
    info: InputInterfaceInfo,
    terminal_link: TermId,
    still_capture_method: StillCaptureMethod,
    trigger_support: TriggerSupport,
    trigger_usage: TriggerUsage,
    format_controls: Vec<PerFormatControls>,
}

#[derive(Debug)]
pub struct OutputHeader {}

bitflags! {
    pub struct InputInterfaceInfo: u8 {
        const DYNAMIC_FORMAT_CHANGE_SUPPORTED = 1 << 0;
    }
}

primitive_enum! {
    pub enum StillCaptureMethod: u8 {
        None = 0,
        Method1 = 1,
        Method2 = 2,
        Method3 = 3,
    }
}

primitive_enum! {
    pub enum TriggerSupport: u8 {
        NotSupported = 0,
        Supported = 1,
    }
}

primitive_enum! {
    pub enum TriggerUsage: u8 {
        InitiateStillImageCapture = 0,
        GeneralPurposeButtonEvent = 1,
    }
}

bitflags! {
    pub struct PerFormatControls: u32 {
        const KEY_FRAME_RATE = 1 << 0;
        const P_FRAME_RATE = 1 << 1;
        const COMP_QUALITY = 1 << 2;
        const COMP_WINDOW_SIZE = 1 << 3;

        const GENERATE_KEY_FRAME = 1 << 4;
        const UPDATE_FRAME_SEGMENT = 1 << 5;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FormatIndex(pub(crate) u8);

#[derive(Debug, Clone, Copy)]
pub struct FrameIndex(pub(crate) u8);

#[derive(Debug)]
pub struct Format {
    format_index: FormatIndex,
    num_frame_descriptors: u8,
    kind: FormatKind,
}

impl Format {
    pub fn index(&self) -> FormatIndex {
        self.format_index
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum FormatKind {
    Uncompressed(FormatUncompressed),
}

#[derive(Debug)]
pub struct FormatUncompressed {
    format: Uuid,
    bits_per_pixel: u8,
    default_frame_index: FrameIndex,
    aspect_ratio_x: u8,
    aspect_ratio_y: u8,
    interlace_flags: InterlaceFlags,
    copy_protect: u8, // cute
}

bitflags! {
    pub struct InterlaceFlags: u8 {
        const INTERLACED = 1 << 0;
        const SINGLE_FIELD_PER_FRAME = 1 << 1;
        const FIELD_1_FIRST = 1 << 2;
        const FIELD_PATTERN_MASK = 0b110000;
    }
}

#[derive(Debug)]
pub struct Frame {
    frame_index: FrameIndex,
    kind: FrameKind,
}

impl Frame {
    pub fn index(&self) -> FrameIndex {
        self.frame_index
    }

    pub fn as_frame_uncompressed(&self) -> Option<&FrameUncompressed> {
        match &self.kind {
            FrameKind::Uncompressed(f) => Some(f),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum FrameKind {
    Uncompressed(FrameUncompressed),
}

#[derive(Debug)]
pub struct FrameUncompressed {
    capabilities: UncompressedFrameCapabilities,
    width: u16,
    height: u16,
    min_bit_rate: u32,
    max_bit_rate: u32,
    max_video_frame_buffer_size: u32,
    default_frame_interval: Duration,
    frame_interval: SupportedFrameIntervals,
}

impl FrameUncompressed {
    pub fn default_frame_interval(&self) -> Duration {
        self.default_frame_interval
    }
}

bitflags! {
    pub struct UncompressedFrameCapabilities: u8 {
        const STILL_IMAGE_SUPPORTED = 1 << 0;
        const FIXED_FRAME_RATE = 1 << 1;
    }
}

#[derive(Debug)]
pub enum SupportedFrameIntervals {
    Continuous {
        min_frame_interval: Duration,
        max_frame_interval: Duration,
        frame_interval_step: Duration,
    },

    Discrete {
        supported_frame_intervals: Vec<Duration>,
    },
}
