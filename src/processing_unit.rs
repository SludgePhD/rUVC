use crate::{
    control::ControlValue,
    topo::{ProcessingUnitDesc, ProcessingUnitId},
    Request, Result, UvcDevice,
};

/// Grants access to a processing unit.
pub struct ProcessingUnit<'a> {
    device: &'a UvcDevice,
    desc: &'a ProcessingUnitDesc,
}

impl<'a> ProcessingUnit<'a> {
    pub(crate) fn new(device: &'a UvcDevice, id: ProcessingUnitId) -> Self {
        let desc = device.topology().processing_unit_by_id(id);

        Self { device, desc }
    }

    pub fn read_control<C: ProcessingUnitControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetCur, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn read_control_min<C: ProcessingUnitControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetMin, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn read_control_max<C: ProcessingUnitControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetMax, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn read_control_res<C: ProcessingUnitControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetRes, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn read_control_default<C: ProcessingUnitControl>(&self) -> Result<C::Value> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        self.read_control_raw(C::ID, Request::GetDef, buf.as_mut())?;
        Ok(<C::Value>::decode(buf.as_mut()))
    }

    pub fn set_control<C: ProcessingUnitControl>(&mut self, value: C::Value) -> Result<()> {
        let mut buf = <<C::Value as ControlValue>::Buf>::default();
        value.encode(buf.as_mut());
        self.set_control_raw(C::ID, buf.as_mut())
    }

    fn set_control_raw(&mut self, control: ControlId, value: &[u8]) -> Result<()> {
        self.device
            .set_entity(self.desc.id().as_raw(), control as _, value)
    }

    fn read_control_raw(&self, control: ControlId, request: Request, buf: &mut [u8]) -> Result<()> {
        self.device
            .read_entity(self.desc.id().as_raw(), request, control as _, buf)
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ControlId {
    #[allow(dead_code)]
    Undefined = 0x00,
    BacklightCompensation = 0x01,
    Brightness = 0x02,
    Contrast = 0x03,
    Gain = 0x04,
    PowerLineFrequency = 0x05,
    Hue = 0x06,
    Saturation = 0x07,
    Sharpness = 0x08,
    Gamma = 0x09,
    WhiteBalanceTemperature = 0x0A,
    WhiteBalanceTemperatureAuto = 0x0B,
    WhiteBalanceComponent = 0x0C,
    WhiteBalanceComponentAuto = 0x0D,
    DigitalMultiplier = 0x0E,
    DigitalMultiplierLimit = 0x0F,
    HueAuto = 0x10,
    AnalogVideoStandard = 0x11,
    AnalogVideoLockStatus = 0x12,
    ContrastAuto = 0x13,
}

pub trait ProcessingUnitControl {
    // TODO seal
    type Value: ControlValue;
    const ID: ControlId;
}

pub struct BacklightCompensation;
impl ProcessingUnitControl for BacklightCompensation {
    type Value = u16;
    const ID: ControlId = ControlId::BacklightCompensation;
}

pub struct Brightness;
impl ProcessingUnitControl for Brightness {
    type Value = i16;
    const ID: ControlId = ControlId::Brightness;
}

pub struct Contrast;
impl ProcessingUnitControl for Contrast {
    type Value = u16;
    const ID: ControlId = ControlId::Contrast;
}

pub struct Gain;
impl ProcessingUnitControl for Gain {
    type Value = u16;
    const ID: ControlId = ControlId::Gain;
}

pub struct PowerLineFrequency;
impl ProcessingUnitControl for PowerLineFrequency {
    type Value = crate::control::PowerLineFrequency;
    const ID: ControlId = ControlId::PowerLineFrequency;
}

pub struct Hue;
impl ProcessingUnitControl for Hue {
    type Value = i16;
    const ID: ControlId = ControlId::Hue;
}

pub struct HueAuto;
impl ProcessingUnitControl for HueAuto {
    type Value = u8;
    const ID: ControlId = ControlId::HueAuto;
}

pub struct Saturation;
impl ProcessingUnitControl for Saturation {
    type Value = u16;
    const ID: ControlId = ControlId::Saturation;
}

pub struct Sharpness;
impl ProcessingUnitControl for Sharpness {
    type Value = u16;
    const ID: ControlId = ControlId::Sharpness;
}

pub struct Gamma;
impl ProcessingUnitControl for Gamma {
    type Value = u16;
    const ID: ControlId = ControlId::Gamma;
}

pub struct WhiteBalanceTemperature;
impl ProcessingUnitControl for WhiteBalanceTemperature {
    type Value = u16;
    const ID: ControlId = ControlId::WhiteBalanceTemperature;
}

pub struct WhiteBalanceTemperatureAuto;
impl ProcessingUnitControl for WhiteBalanceTemperatureAuto {
    type Value = u8;
    const ID: ControlId = ControlId::WhiteBalanceTemperatureAuto;
}

pub struct WhiteBalanceComponent;
impl ProcessingUnitControl for WhiteBalanceComponent {
    type Value = crate::control::WhiteBalanceComponents;
    const ID: ControlId = ControlId::WhiteBalanceComponent;
}

pub struct WhiteBalanceComponentAuto;
impl ProcessingUnitControl for WhiteBalanceComponentAuto {
    type Value = u8;
    const ID: ControlId = ControlId::WhiteBalanceComponentAuto;
}
