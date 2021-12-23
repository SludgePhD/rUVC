use crate::{
    control::ControlValue,
    topo::{CameraId, CameraTerminalDesc},
    Request, Result, UvcDevice,
};

/// Grants access to a camera input terminal.
pub struct CameraTerminal<'a> {
    device: &'a UvcDevice,
    id: CameraId,
    desc: &'a CameraTerminalDesc,
}

impl<'a> CameraTerminal<'a> {
    pub(crate) fn new(device: &'a UvcDevice, id: CameraId) -> Self {
        // side-effect: validates `id`
        let desc = device.topology().camera_terminal_by_id(id);

        Self { device, id, desc }
    }

    pub fn read_control<C: CameraControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetCur, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn read_control_min<C: CameraControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetMin, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn read_control_max<C: CameraControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetMax, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn read_control_res<C: CameraControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetRes, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn read_control_default<C: CameraControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetDef, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn set_control<C: CameraControl>(&mut self, value: C::Value) -> Result<()> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        value.encode(buf.as_mut());
        self.set_control_raw(C::ID, buf.as_mut())
    }

    fn set_control_raw(&mut self, control: ControlId, value: &[u8]) -> Result<()> {
        self.device
            .set_entity(self.id.as_raw(), control as _, value)
    }

    fn read_control_raw(&self, control: ControlId, req: Request, buf: &mut [u8]) -> Result<()> {
        self.device
            .read_entity(self.id.as_raw(), req, control as _, buf)
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ControlId {
    Undefined = 0x00,
    ScanningMode = 0x01,
    AutoExposureMode = 0x02,
    AutoExposurePriority = 0x03,
    ExposureTimeAbs = 0x04,
    ExposureTimeRel = 0x05,
    FocusAbs = 0x06,
    FocusRel = 0x07,
    FocusAuto = 0x08,
    IrisAbs = 0x09,
    IrisRel = 0x0A,
    ZoomAbs = 0x0B,
    ZoomRel = 0x0C,
    PanTiltAbs = 0x0D,
    PanTiltRel = 0x0E,
    RollAbs = 0x0F,
    RollRel = 0x10,
    Privacy = 0x11,
    FocusSimple = 0x12,
    Window = 0x13,
    RegionOfInterest = 0x14,
}

pub trait CameraControl {
    // TODO seal
    type Value: ControlValue;
    const ID: ControlId;
}

pub struct ScanningMode;
impl CameraControl for ScanningMode {
    type Value = bool;
    const ID: ControlId = ControlId::ScanningMode;
}

pub struct AutoExposureMode;
impl CameraControl for AutoExposureMode {
    type Value = crate::control::AutoExposureMode;
    const ID: ControlId = ControlId::AutoExposureMode;
}

pub struct AutoExposurePriority;
impl CameraControl for AutoExposurePriority {
    type Value = u8;
    const ID: ControlId = ControlId::AutoExposurePriority;
}

pub struct ExposureTimeAbs;
impl CameraControl for ExposureTimeAbs {
    type Value = u32;
    const ID: ControlId = ControlId::ExposureTimeAbs;
}

pub struct ExposureTimeRel;
impl CameraControl for ExposureTimeRel {
    type Value = i8;
    const ID: ControlId = ControlId::ExposureTimeRel;
}

pub struct FocusAbs;
impl CameraControl for FocusAbs {
    type Value = u16;
    const ID: ControlId = ControlId::FocusAbs;
}

pub struct FocusRel;
impl CameraControl for FocusRel {
    type Value = crate::control::FocusRel;
    const ID: ControlId = ControlId::FocusRel;
}

pub struct FocusSimple;
impl CameraControl for FocusSimple {
    type Value = crate::control::FocusSimple;
    const ID: ControlId = ControlId::FocusSimple;
}

pub struct FocusAuto;
impl CameraControl for FocusAuto {
    type Value = bool;
    const ID: ControlId = ControlId::FocusAuto;
}

pub struct IrisAbs;
impl CameraControl for IrisAbs {
    type Value = u16; // fstop * 100 TODO newtype?
    const ID: ControlId = ControlId::IrisAbs;
}

pub struct IrisRel;
impl CameraControl for IrisRel {
    type Value = u8; // TODO newtype
    const ID: ControlId = ControlId::IrisRel;
}

pub struct ZoomAbs;
impl CameraControl for ZoomAbs {
    type Value = u16;
    const ID: ControlId = ControlId::ZoomAbs;
}
