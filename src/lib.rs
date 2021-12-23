//! TODO: write docs
//!
//! Dealing with a device entity `Ent`:
//! - `EntDesc` describes an entity's static properties, read from the device descriptor.
//! - `EntId` is a small `Copy` type that refers to an `EntDesc`.
//! - `Ent<'a>` grants access to the entity's properties, borrowing the opened device.

#[macro_use]
mod util;
pub mod camera;
pub mod control;
mod detect;
mod error;
pub mod processing_unit;
pub mod streaming_interface;
pub mod topo;

use std::{fmt, time::Duration};

use camera::CameraTerminal;
use detect::UvcInfo;
pub use error::Error;
use error::*;
use processing_unit::ProcessingUnit;
use rusb::{Context, Device, DeviceHandle, UsbContext};
use streaming_interface::StreamingInterface;
use topo::{CameraId, ProcessingUnitId, StreamingInterfaceDesc, StreamingInterfaceId, Topology};

pub type Result<T> = std::result::Result<T, Error>;

/// Identifies a UVC device.
pub struct UvcDeviceDesc {
    usb: Device<Context>,
    uvc_info: UvcInfo,
}

impl UvcDeviceDesc {
    pub fn vendor_id(&self) -> u16 {
        // unwrap: always succeeds
        self.usb.device_descriptor().unwrap().vendor_id()
    }

    pub fn product_id(&self) -> u16 {
        // unwrap: always succeeds
        self.usb.device_descriptor().unwrap().product_id()
    }

    pub fn open(self) -> Result<UvcDevice> {
        UvcDevice::open(self)
    }
}

impl fmt::Debug for UvcDeviceDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UvcDeviceDesc")
            .field("uvc_info", &self.uvc_info)
            .finish()
    }
}

pub fn list() -> Result<impl Iterator<Item = UvcDeviceDesc>> {
    let ctx = Context::new().during(Action::EnumeratingDevices)?;
    let list = ctx.devices().during(Action::EnumeratingDevices)?;

    let devices = list
        .iter()
        .filter_map(|dev| match detect::detect_uvc(&dev) {
            Ok(Some(info)) => Some(UvcDeviceDesc {
                usb: dev,
                uvc_info: info,
            }),
            Ok(None) => None,
            Err(e) => {
                log::error!("{:?}: {}", dev, e);
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(devices.into_iter())
}

pub struct UvcDevice {
    usb: DeviceHandle<Context>,
    uvc_info: UvcInfo,
    timeout: Duration,
}

impl UvcDevice {
    fn open(desc: UvcDeviceDesc) -> Result<Self> {
        let mut usb = desc.usb.open().during(Action::OpeningDevice)?;
        if let Err(e) = usb.set_auto_detach_kernel_driver(true) {
            log::warn!("set_auto_detach_kernel_driver failed: {}", e);
        }

        let config = usb.active_configuration().during(Action::OpeningDevice)?;
        if config != 1 {
            usb.set_active_configuration(1)
                .during(Action::OpeningDevice)?;
        }

        usb.claim_interface(desc.uvc_info.control_interface.interface_number)
            .during(Action::OpeningDevice)?;
        for intf in &desc.uvc_info.streaming_interfaces {
            usb.claim_interface(intf.id().0)
                .during(Action::OpeningDevice)?;
        }

        let config = usb.active_configuration().during(Action::OpeningDevice)?;
        if config != 1 {
            return err("failed to claim device", Action::OpeningDevice);
        }

        Ok(UvcDevice {
            usb,
            uvc_info: desc.uvc_info,
            timeout: Duration::from_millis(1000),
        })
    }

    fn with_usb<T>(&self, mut cb: impl FnMut(&DeviceHandle<Context>) -> Result<T>) -> Result<T> {
        // On the Leap Motion, one of the first transfers might time out (can be read or write,
        // depending on the exact sequence of transfers performed). Not sure why, but this works
        // around that.

        match cb(&self.usb) {
            Err(e) if e.is_usb_timeout() => {
                log::warn!("USB timeout, retrying request");
                cb(&self.usb)
            }
            other => other,
        }
    }

    /// Performs a `SET_CUR` request on an "entity" control (eg. an input, output, or unit's control).
    fn set_entity(&self, entity_id: u8, cs: u8, data: &[u8]) -> Result<()> {
        self.set_interface_entity(
            self.uvc_info.control_interface.interface_number,
            entity_id,
            cs,
            data,
        )
    }

    fn set_interface_entity(
        &self,
        interface: u8,
        entity_id: u8,
        cs: u8,
        data: &[u8],
    ) -> Result<()> {
        const SET_ENTITY_REQ: u8 = 0b00100001;

        let value = u16::from(cs) << 8;
        let index = u16::from(entity_id) << 8 | u16::from(interface);
        self.with_usb(|usb| {
            usb.write_control(
                SET_ENTITY_REQ,
                Request::SetCur as _,
                value,
                index,
                data,
                self.timeout,
            )
            .during(Action::WritingControl)?;
            Ok(())
        })
    }

    fn read_entity(&self, entity_id: u8, request: Request, cs: u8, buf: &mut [u8]) -> Result<()> {
        self.read_interface_entity(
            self.uvc_info.control_interface.interface_number,
            entity_id,
            request,
            cs,
            buf,
        )
    }

    fn read_interface_entity(
        &self,
        interface: u8,
        entity_id: u8,
        request: Request,
        cs: u8,
        buf: &mut [u8],
    ) -> Result<()> {
        const GET_ENTITY_REQ: u8 = 0b10100001;

        let value = u16::from(cs) << 8;
        let index = u16::from(entity_id) << 8 | u16::from(interface);

        self.with_usb(|usb| {
            usb.read_control(
                GET_ENTITY_REQ,
                request as _,
                value,
                index,
                buf,
                self.timeout,
            )
            .during(Action::ReadingControl)?;
            Ok(())
        })
    }

    pub fn read_manufacturer_string(&self) -> Result<String> {
        Ok(self
            .usb
            .read_manufacturer_string_ascii(&self.usb.device().device_descriptor().unwrap())
            .during(Action::ReadingDeviceString)?)
    }

    pub fn read_product_string(&self) -> Result<String> {
        Ok(self
            .usb
            .read_product_string_ascii(&self.usb.device().device_descriptor().unwrap())
            .during(Action::ReadingDeviceString)?)
    }

    pub fn topology(&self) -> &Topology {
        &self.uvc_info.control_interface.topo
    }

    /// Returns the device's streaming interfaces.
    ///
    /// Streaming interfaces transport video data over the USB channel (either from the device to
    /// the host, or from the host to the device).
    pub fn streaming_interfaces(&self) -> &[StreamingInterfaceDesc] {
        &self.uvc_info.streaming_interfaces
    }

    pub fn streaming_interface_by_id(&self, id: StreamingInterfaceId) -> StreamingInterface<'_> {
        StreamingInterface::new(self, id)
    }

    pub fn camera_terminal_by_id(&self, id: CameraId) -> CameraTerminal<'_> {
        CameraTerminal::new(self, id)
    }

    pub fn processing_unit_by_id(&self, id: ProcessingUnitId) -> ProcessingUnit<'_> {
        ProcessingUnit::new(self, id)
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum Request {
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
