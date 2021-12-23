use std::{
    io::{self, Read},
    time::Duration,
};

use crate::{
    control::{ControlValue, ProbeCommitControls},
    error::{Action, ResultExt},
    topo::{FormatIndex, FrameIndex, StreamingInterfaceDesc, StreamingInterfaceId},
    Request, Result, UvcDevice,
};

pub struct StreamingInterface<'a> {
    device: &'a UvcDevice,
    desc: &'a StreamingInterfaceDesc,
}

impl<'a> StreamingInterface<'a> {
    pub(crate) fn new(device: &'a UvcDevice, id: StreamingInterfaceId) -> Self {
        let desc = device
            .streaming_interfaces()
            .iter()
            .find(|i| i.id().0 == id.0)
            .unwrap();

        Self { device, desc }
    }

    pub fn start_stream(&mut self, format: FormatIndex, frame: FrameIndex) -> Result<Stream<'_>> {
        self.negotiate_stream_params(format, frame)?;
        Ok(self.start_stream_no_negotiate())
    }

    pub fn start_stream_no_negotiate(&mut self) -> Stream<'_> {
        Stream {
            device: self.device,
            ep: self.desc.endpoint_address(),
        }
    }

    fn negotiate_stream_params(
        &mut self,
        format_index: FormatIndex,
        frame_index: FrameIndex,
    ) -> Result<()> {
        let frame = self.desc.frame_by_index(frame_index);
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
        self.set_control::<Probe>(controls)?;
        let controls = self.read_control::<Probe>()?;
        log::debug!("final parameters: {:?}", controls);
        self.set_control::<Commit>(controls)?;
        Ok(())
    }

    pub fn read_control<C: StreamingControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetCur, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn read_control_min<C: StreamingControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetMin, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn read_control_max<C: StreamingControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetMax, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn set_control<C: StreamingControl>(&mut self, value: C::Value) -> Result<()> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        value.encode(buf.as_mut());
        self.set_control_raw(C::ID, buf.as_mut())
    }

    fn set_control_raw(&mut self, control: ControlId, value: &[u8]) -> Result<()> {
        self.device
            .set_interface_entity(self.desc.id().0, 0, control as _, value)
    }

    fn read_control_raw(&self, control: ControlId, req: Request, buf: &mut [u8]) -> Result<()> {
        self.device
            .read_interface_entity(self.desc.id().0, 0, req, control as _, buf)
    }
}

pub struct Stream<'a> {
    device: &'a UvcDevice,
    ep: u8,
}

impl Read for Stream<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.device
            .with_usb(|usb| {
                usb.read_bulk(self.ep, buf, self.device.timeout)
                    .during(Action::StreamRead)
            })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

/// Controls associated with Video Streaming Interfaces.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ControlId {
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

pub trait StreamingControl {
    // TODO seal
    type Value: ControlValue;
    const ID: ControlId;
}

pub struct Probe;
impl StreamingControl for Probe {
    type Value = ProbeCommitControls;
    const ID: ControlId = ControlId::Probe;
}

pub struct Commit;
impl StreamingControl for Commit {
    type Value = ProbeCommitControls;
    const ID: ControlId = ControlId::Commit;
}
