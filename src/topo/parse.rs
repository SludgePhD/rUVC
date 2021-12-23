use std::io;

use byteorder::{ReadBytesExt, LE};
use rusb::InterfaceDescriptor;

use crate::{
    error::*,
    util::{io_err, io_err_res, split_descriptors, BcdVersion, BytesExt},
    Result,
};

use super::*;

/// The value of `bDescriptorType` of all descriptors we're interested in.
const VIDEO_INTERFACE_DESC_TYPE: u8 = 36;

const CONTROL_DESC_SUBTYPE_HEADER: u8 = 0x01;
const CONTROL_DESC_SUBTYPE_INPUT_TERM: u8 = 0x02;
const CONTROL_DESC_SUBTYPE_OUTPUT_TERMINAL: u8 = 0x03;
const CONTROL_DESC_SUBTYPE_SELECTOR_UNIT: u8 = 0x04;
const CONTROL_DESC_SUBTYPE_PROCESSING_UNIT: u8 = 0x05;
const CONTROL_DESC_SUBTYPE_EXTENSION_UNIT: u8 = 0x06;
const CONTROL_DESC_SUBTYPE_ENCODING_UNIT: u8 = 0x07;

const STREAM_DESC_SUBTYPE_INPUT_HEADER: u8 = 0x01;
const STREAM_DESC_SUBTYPE_OUTPUT_HEADER: u8 = 0x02;
const STREAM_DESC_SUBTYPE_STILL_IMAGE_FRAME: u8 = 0x03;
const STREAM_DESC_SUBTYPE_FORMAT_UNCOMPRESSED: u8 = 0x04;
const STREAM_DESC_SUBTYPE_FRAME_UNCOMPRESSED: u8 = 0x05;
const STREAM_DESC_SUBTYPE_FORMAT_MJPEG: u8 = 0x06;
const STREAM_DESC_SUBTYPE_FRAME_MJPEG: u8 = 0x07;
const STREAM_DESC_SUBTYPE_FORMAT_MPEG2TS: u8 = 0x0A;
const STREAM_DESC_SUBTYPE_FORMAT_DV: u8 = 0x0C;
const STREAM_DESC_SUBTYPE_COLORFORMAT: u8 = 0x0D;
const STREAM_DESC_SUBTYPE_FORMAT_FRAME_BASED: u8 = 0x10;
const STREAM_DESC_SUBTYPE_FRAME_FRAME_BASED: u8 = 0x11;
const STREAM_DESC_SUBTYPE_FORMAT_STREAM_BASED: u8 = 0x12;
const STREAM_DESC_SUBTYPE_FORMAT_H264: u8 = 0x13;
const STREAM_DESC_SUBTYPE_FRAME_H264: u8 = 0x14;
const STREAM_DESC_SUBTYPE_FORMAT_H264_SIMULCAST: u8 = 0x15;
const STREAM_DESC_SUBTYPE_FORMAT_VP8: u8 = 0x16;
const STREAM_DESC_SUBTYPE_FRAME_VP8: u8 = 0x17;
const STREAM_DESC_SUBTYPE_FORMAT_VP8_SIMULCAST: u8 = 0x18;

pub(crate) fn parse_control_desc(desc: &InterfaceDescriptor<'_>) -> Result<Topology> {
    let mut parser = ControlDescParser {
        header: None,
        units: Vec::new(),
        inputs: Vec::new(),
        outputs: Vec::new(),
    };
    for (ty, data) in split_descriptors(desc.extra()) {
        if ty == VIDEO_INTERFACE_DESC_TYPE {
            parser
                .parse_descriptor(&data[2..])
                .during(Action::AccessingDeviceDescriptor)?;
        } else {
            log::debug!("skipping descriptor of type {}", ty);
        }
    }

    // FIXME: the interrupt endpoint descriptor also carries custom data according to the spec,
    // however it is absent on the Leap Motion (or `lsusb` doesn't display it)

    let header = match parser.header {
        Some(header) => header,
        None => {
            return err(
                "missing VC_HEADER descriptor",
                Action::AccessingDeviceDescriptor,
            );
        }
    };

    Ok(Topology {
        header,
        units: parser.units,
        inputs: parser.inputs,
        outputs: parser.outputs,
    })
}

struct ControlDescParser {
    header: Option<ControlHeader>,
    units: Vec<UnitDesc>,
    inputs: Vec<InputTerminalDesc>,
    outputs: Vec<OutputTerminalDesc>,
}

impl ControlDescParser {
    fn parse_descriptor(&mut self, raw: &[u8]) -> io::Result<()> {
        match self.parse_descriptor_impl(raw) {
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                log::warn!(
                    "UVC descriptor too short, please report a bug to the device manufacturer"
                );
                log::debug!("retrying with 100 extra zero bytes");
                log::debug!("descriptor data: {:02x?}", raw);

                let mut buf = vec![0; raw.len() + 100];
                buf[..raw.len()].copy_from_slice(raw);

                self.parse_descriptor_impl(&buf)
            }
            res => res,
        }
    }

    fn parse_descriptor_impl(&mut self, mut raw: &[u8]) -> io::Result<()> {
        let subtype = raw.read_u8()?;
        match subtype {
            CONTROL_DESC_SUBTYPE_HEADER => {
                if self.header.is_some() {
                    return io_err_res("duplicate VC_HEADER descriptor");
                }

                self.header = Some(ControlHeader {
                    uvc_version: BcdVersion(raw.read_u16::<LE>()?),
                    total_len: raw.read_u16::<LE>()?,
                    clock_freq_hz: raw.read_u32::<LE>()?,
                    streaming_interfaces: {
                        let count = raw.read_u8()?;
                        (0..count)
                            .map(|_| raw.read_u8())
                            .collect::<io::Result<Vec<_>>>()?
                    },
                });

                Ok(())
            }
            CONTROL_DESC_SUBTYPE_INPUT_TERM => {
                let mut term = InputTerminalDesc {
                    term_id: TermId::new(raw.read_u8()?).ok_or_else(|| {
                        io_err("bTerminalID is 0, only non-zero numbers are allowed")
                    })?,
                    term_type: raw.read_u16::<LE>()?,
                    assoc: TermId::new(raw.read_u8()?),
                    string: raw.read_u8()?,
                    kind: InputTerminalKind::Other,
                };
                if term.terminal_type() == Some(InputTerminalType::InCamera) {
                    term.kind = InputTerminalKind::Camera(CameraTerminalDesc {
                        objective_focal_length_min: raw.read_u16::<LE>()?,
                        objective_focal_length_max: raw.read_u16::<LE>()?,
                        ocular_focal_length: raw.read_u16::<LE>()?,
                        controls: CameraControls::from_bits_truncate(
                            raw.read_length_prefixed_bitmask()?,
                        ),
                    });
                }

                self.inputs.push(term);

                Ok(())
            }
            CONTROL_DESC_SUBTYPE_OUTPUT_TERMINAL => {
                self.outputs.push(OutputTerminalDesc {
                    term_id: raw.read_nonzero_term_id()?,
                    term_type: raw.read_u16::<LE>()?,
                    assoc: TermId::new(raw.read_u8()?),
                    source: raw.read_nonzero_source_id()?,
                    string: raw.read_u8()?,
                });
                Ok(())
            }
            CONTROL_DESC_SUBTYPE_SELECTOR_UNIT => {
                self.units.push(UnitDesc {
                    kind: UnitKind::Selector(SelectorUnitDesc {
                        id: SelectorUnitId(raw.read_nonzero_unit_id()?),
                        inputs: {
                            let num = raw.read_u8()?;
                            (0..num)
                                .map(|_| raw.read_nonzero_source_id())
                                .collect::<io::Result<Vec<_>>>()?
                        },
                    }),
                });
                Ok(())
            }
            CONTROL_DESC_SUBTYPE_PROCESSING_UNIT => {
                // The Leap Motion (fw 1.7.0 and older) has a bug where this descriptor only has a
                // total length of 12 (including the length and descriptor type bytes), but it needs
                // to have length 13 to be valid.
                // It looks like `lsusb` will just keep reading past the descriptor and interpret
                // the length byte (28 -> 0x1c) of the next descriptor as the `standards` field.
                // In our case, this is handled by the `parse_descriptor` fallback.

                self.units.push(UnitDesc {
                    kind: UnitKind::Processing(ProcessingUnitDesc {
                        id: ProcessingUnitId(raw.read_nonzero_unit_id()?),
                        source: raw.read_nonzero_source_id()?,
                        max_multiplier: raw.read_u16::<LE>()?,
                        controls: ProcessingUnitControls::from_bits_truncate(
                            raw.read_length_prefixed_bitmask()?,
                        ),
                        string: raw.read_u8()?,
                        standards: VideoStandards::from_bits_truncate(raw.read_u8()?),
                    }),
                });
                Ok(())
            }
            CONTROL_DESC_SUBTYPE_EXTENSION_UNIT => {
                self.units.push(UnitDesc {
                    kind: UnitKind::Extension(ExtensionUnitDesc {
                        id: ExtensionUnitId(raw.read_nonzero_unit_id()?),
                        extension_code: raw.read_guid()?,
                        num_controls: raw.read_u8()?,
                        inputs: {
                            let count = raw.read_u8()?;
                            (0..count)
                                .map(|_| raw.read_nonzero_source_id())
                                .collect::<io::Result<Vec<_>>>()?
                        },
                        controls_bitmap: {
                            let size = raw.read_u8()?;
                            (0..size)
                                .map(|_| raw.read_u8())
                                .collect::<io::Result<Vec<_>>>()?
                        },
                    }),
                });
                Ok(())
            }
            CONTROL_DESC_SUBTYPE_ENCODING_UNIT => {
                // TODO
                io_err_res(format!("unimplemented descriptor subtype {}", subtype))
            }
            _ => io_err_res(format!("invalid/unknown descriptor subtype {}", subtype)),
        }
    }
}

pub(crate) fn parse_streaming_descriptor(
    desc: &InterfaceDescriptor<'_>,
) -> Result<StreamingInterfaceDesc> {
    let mut parser = StreamingDescParser {
        in_header: None,
        out_header: None,
        formats: Vec::new(),
        frames: Vec::new(),
    };

    for (ty, data) in split_descriptors(desc.extra()) {
        if ty == VIDEO_INTERFACE_DESC_TYPE {
            parser
                .parse_descriptor(&data[2..])
                .during(Action::AccessingDeviceDescriptor)?;
        } else {
            log::debug!("skipping descriptor of type {}", ty);
        }
    }

    Ok(StreamingInterfaceDesc {
        id: StreamingInterfaceId(desc.interface_number()),
        kind: match (parser.in_header, parser.out_header) {
            (None, Some(output)) => StreamingInterfaceKind::Output(output),
            (Some(input), None) => StreamingInterfaceKind::Input(input),
            (None, None) => {
                return err(
                    "missing header in Video Streaming interface",
                    Action::AccessingDeviceDescriptor,
                )
            }
            (Some(_), Some(_)) => {
                return err(
                    "Video Streaming interface has both input and output descriptor",
                    Action::AccessingDeviceDescriptor,
                )
            }
        },
        formats: parser.formats,
        frames: parser.frames,
    })
}

struct StreamingDescParser {
    in_header: Option<InputHeader>,
    out_header: Option<OutputHeader>,
    formats: Vec<Format>,
    frames: Vec<Frame>,
}

impl StreamingDescParser {
    fn parse_descriptor(&mut self, raw: &[u8]) -> io::Result<()> {
        match self.parse_descriptor_impl(raw) {
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                log::warn!(
                    "UVC Video Streaming interface descriptor too short, please report a bug to the device manufacturer"
                );
                log::debug!("retrying with 100 extra zero bytes");
                log::debug!("descriptor data: {:#04x?}", raw);

                let mut buf = vec![0; raw.len() + 100];
                buf[..raw.len()].copy_from_slice(raw);

                self.parse_descriptor_impl(&buf)
            }
            res => res,
        }
    }

    fn parse_descriptor_impl(&mut self, mut raw: &[u8]) -> io::Result<()> {
        let subtype = raw.read_u8()?;
        match subtype {
            STREAM_DESC_SUBTYPE_INPUT_HEADER => {
                if self.in_header.is_some() {
                    return io_err_res("duplicate input header descriptor");
                }

                let num_formats = raw.read_u8()?;
                self.in_header = Some(InputHeader {
                    num_formats,
                    total_length: raw.read_u16::<LE>()?,
                    endpoint_address: raw.read_u8()?,
                    info: InputInterfaceInfo::from_bits_truncate(raw.read_u8()?),
                    terminal_link: raw.read_nonzero_term_id()?,
                    still_capture_method: {
                        let raw = raw.read_u8()?;
                        StillCaptureMethod::from_raw(raw).unwrap_or_else(|| {
                            log::warn!("invalid value {} for `bStillCaptureMethod`", raw);
                            StillCaptureMethod::None
                        })
                    },
                    trigger_support: {
                        let raw = raw.read_u8()?;
                        TriggerSupport::from_raw(raw).unwrap_or_else(|| {
                            log::warn!("invalid value {} for `bTriggerSupport`", raw);
                            TriggerSupport::NotSupported
                        })
                    },
                    trigger_usage: {
                        let raw = raw.read_u8()?;
                        TriggerUsage::from_raw(raw).unwrap_or_else(|| {
                            log::warn!("invalid value {} for `bTriggerUsage`", raw);
                            TriggerUsage::InitiateStillImageCapture
                        })
                    },
                    format_controls: {
                        let control_size = raw.read_u8()?;

                        // This is `num_format` units with `control_size` bytes each.
                        (0..num_formats)
                            .map(|_| {
                                raw.read_bitmask(control_size)
                                    .map(|bits| PerFormatControls::from_bits_truncate(bits))
                            })
                            .collect::<io::Result<Vec<_>>>()?
                    },
                });
                Ok(())
            }
            STREAM_DESC_SUBTYPE_FORMAT_UNCOMPRESSED => {
                self.formats.push(Format {
                    format_index: FormatIndex(raw.read_u8()?),
                    num_frame_descriptors: raw.read_u8()?,
                    kind: FormatKind::Uncompressed(FormatUncompressed {
                        format: raw.read_guid()?,
                        bits_per_pixel: raw.read_u8()?,
                        default_frame_index: FrameIndex(raw.read_u8()?),
                        aspect_ratio_x: raw.read_u8()?,
                        aspect_ratio_y: raw.read_u8()?,
                        interlace_flags: InterlaceFlags::from_bits_truncate(raw.read_u8()?),
                        copy_protect: raw.read_u8()?,
                    }),
                });
                Ok(())
            }
            STREAM_DESC_SUBTYPE_FRAME_UNCOMPRESSED => {
                self.frames.push(Frame {
                    frame_index: FrameIndex(raw.read_u8()?),
                    kind: FrameKind::Uncompressed(FrameUncompressed {
                        capabilities: UncompressedFrameCapabilities::from_bits_truncate(
                            raw.read_u8()?,
                        ),
                        width: raw.read_u16::<LE>()?,
                        height: raw.read_u16::<LE>()?,
                        min_bit_rate: raw.read_u32::<LE>()?,
                        max_bit_rate: raw.read_u32::<LE>()?,
                        max_video_frame_buffer_size: raw.read_u32::<LE>()?,
                        default_frame_interval: raw.read_time_100ns()?,
                        frame_interval: {
                            let ty = raw.read_u8()?;
                            match ty {
                                0 => {
                                    // Continuous
                                    SupportedFrameIntervals::Continuous {
                                        min_frame_interval: raw.read_time_100ns()?,
                                        max_frame_interval: raw.read_time_100ns()?,
                                        frame_interval_step: raw.read_time_100ns()?,
                                    }
                                }
                                n => {
                                    // `n` discrete intervals.
                                    SupportedFrameIntervals::Discrete {
                                        supported_frame_intervals: (0..n)
                                            .map(|_| raw.read_time_100ns())
                                            .collect::<io::Result<Vec<_>>>()?,
                                    }
                                }
                            }
                        },
                    }),
                });
                Ok(())
            }
            STREAM_DESC_SUBTYPE_OUTPUT_HEADER
            | STREAM_DESC_SUBTYPE_STILL_IMAGE_FRAME
            | STREAM_DESC_SUBTYPE_FORMAT_MJPEG
            | STREAM_DESC_SUBTYPE_FRAME_MJPEG
            | STREAM_DESC_SUBTYPE_FORMAT_MPEG2TS
            | STREAM_DESC_SUBTYPE_FORMAT_DV
            | STREAM_DESC_SUBTYPE_COLORFORMAT
            | STREAM_DESC_SUBTYPE_FORMAT_FRAME_BASED
            | STREAM_DESC_SUBTYPE_FRAME_FRAME_BASED
            | STREAM_DESC_SUBTYPE_FORMAT_STREAM_BASED
            | STREAM_DESC_SUBTYPE_FORMAT_H264
            | STREAM_DESC_SUBTYPE_FRAME_H264
            | STREAM_DESC_SUBTYPE_FORMAT_H264_SIMULCAST
            | STREAM_DESC_SUBTYPE_FORMAT_VP8
            | STREAM_DESC_SUBTYPE_FRAME_VP8
            | STREAM_DESC_SUBTYPE_FORMAT_VP8_SIMULCAST => {
                // TODO
                io_err_res(format!("unimplemented descriptor subtype {}", subtype))
            }
            _ => io_err_res(format!("invalid/unknown descriptor subtype {}", subtype)),
        }
    }
}
