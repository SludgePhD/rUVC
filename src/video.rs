//! Video Probe and Commit Controls (4.3.1.1)

use std::{fmt, mem, time::Duration};

use crate::{
    error::{err, Action, ResultExt},
    topo::{FormatIndex, FrameIndex, SourceId, StreamingInterfaceId},
    Result, UvcDevice,
};
use zerocopy::{AsBytes, FromBytes};

#[derive(Default, AsBytes, FromBytes, Debug, Clone, Copy)]
#[repr(C, packed)]
#[allow(non_snake_case)]
struct ProbeCommitControls {
    bmHint: u16,
    bFormatIndex: u8,
    bFrameIndex: u8,
    dwFrameInterval: u32,
    wKeyFrameRate: u16,
    wPFrameRate: u16,
    wCompQuality: u16,
    wCompWindowSize: u16,
    wDelay: u16,
    dwMaxVideoFrameSize: u32,
    dwMaxPayloadTransferSize: u32,
    dwClockFrequency: u32,
    bmFramingInfo: u8,
    bPreferedVersion: u8, // (sic)
    bMinVersion: u8,
    bMaxVersion: u8,
    bUsage: u8,
    bBitDepthLuma: u8,
    bmSettings: u8,
    bMaxNumberOfRefFramesPlus1: u8,
    bmRateControlModes: u16,
    bmLayoutPerStream: u64,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum Request {
    // TODO dedup
    Undefined = 0x00,
    SetCur = 0x01,
    SetCurAll = 0x11,
    GetCur = 0x81,
    GetMin = 0x82,
    GetMax = 0x83,
    GetRes = 0x84,
    GetLen = 0x85,
    GetInfo = 0x86,
    GetDef = 0x87,
    GetCurAll = 0x91,
    GetMinAll = 0x92,
    GetMaxAll = 0x93,
    GetResAll = 0x94,
    GetDefAll = 0x97,
}

/// Controls associated with Video Streaming Interfaces.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum StreamingControl {
    Undefined = 0x00,
    Probe = 0x01,
    Commit = 0x02,
    StillProbe = 0x03,
    StillCommit = 0x04,
    StillImageTrigger = 0x05,
    StreamErrorCode = 0x06,
    GenerateKeyFrame = 0x07,
    UpdateFrameSegment = 0x08,
    SynchDelay = 0x09,
}

impl UvcDevice {
    pub(crate) fn negotiate_stream_params(
        &mut self,
        interface_id: StreamingInterfaceId,
        format_index: FormatIndex,
        frame_index: FrameIndex,
    ) -> Result<()> {
        let interface = self.streaming_interface_by_id(interface_id);
        let frame = interface.frame_by_index(frame_index);
        let interval = frame
            .as_frame_uncompressed()
            .unwrap()
            .default_frame_interval();
        let interval_100ns = interval.as_secs_f64() / Duration::from_nanos(100).as_secs_f64();

        let controls = ProbeCommitControls {
            bFormatIndex: format_index.0,
            bFrameIndex: frame_index.0,
            dwFrameInterval: interval_100ns as u32,
            ..Default::default()
        };
        log::debug!("negotiating parameters: {:?}", controls);
        let accessor = self.streaming(interface_id);
        accessor.set_cur(StreamingControl::Probe, controls.as_bytes())?;
        let mut buf = [0; mem::size_of::<ProbeCommitControls>()];
        accessor.get_cur(StreamingControl::Probe, &mut buf)?;
        let controls =
            ProbeCommitControls::read_from(&buf[..]).expect("couldn't cast to `Control`");
        log::debug!("final parameters: {:?}", controls);
        accessor.set_cur(StreamingControl::Commit, &buf)?;
        Ok(())
    }

    fn streaming(&self, interface: StreamingInterfaceId) -> StreamingInterfaceAccess<'_> {
        StreamingInterfaceAccess {
            device: self,
            interface,
        }
    }
}

struct StreamingInterfaceAccess<'a> {
    device: &'a UvcDevice,
    interface: StreamingInterfaceId,
}

impl StreamingInterfaceAccess<'_> {
    fn set_cur(&self, control: StreamingControl, data: &[u8]) -> Result<()> {
        self.write(Request::SetCur, control, data)
    }

    fn get_cur<'buf>(
        &self,
        control: StreamingControl,
        buf: &'buf mut [u8],
    ) -> Result<&'buf mut [u8]> {
        self.read(Request::GetCur, control, buf)
    }

    fn read<'buf>(
        &self,
        request: Request,
        control: StreamingControl,
        buf: &'buf mut [u8],
    ) -> Result<&'buf mut [u8]> {
        log::trace!("{:?}({:?})", request, control);
        let bytes = self.device.with_usb(|usb| {
            let bytes = usb
                .read_control(
                    REQ_TYPE_GET,
                    request as _,
                    (control as u16) << 8,
                    self.interface.0.into(),
                    buf,
                    self.device.timeout,
                )
                .during(Action::StreamNegotiation)?;

            Ok(bytes)
        })?;
        Ok(&mut buf[..bytes])
    }

    fn write(&self, request: Request, control: StreamingControl, data: &[u8]) -> Result<()> {
        log::trace!("{:?}({:?})", request, control);
        self.device.with_usb(|usb| {
            let bytes = usb
                .write_control(
                    REQ_TYPE_SET,
                    request as _,
                    (control as u16) << 8,
                    self.interface.0.into(),
                    data,
                    self.device.timeout,
                )
                .during(Action::StreamNegotiation)?;
            if bytes != data.len() {
                return err(
                    format!("control write only wrote {}/{} bytes", bytes, data.len()),
                    Action::StreamNegotiation,
                );
            }

            Ok(())
        })
    }
}

const REQ_TYPE_SET: u8 = 0b00100001;
const REQ_TYPE_GET: u8 = 0b10100001;
