use crate::{
    error::*,
    topo::{self, StreamingInterfaceDesc, Topology},
    util::split_descriptors,
    Result,
};
use rusb::{Context, Device, TransferType};
use zerocopy::FromBytes;

const IAD_DEVICE_CLASS: u8 = 0xEF;
const IAD_DEVICE_SUBCLASS: u8 = 0x02;
const IAD_DEVICE_PROTOCOL: u8 = 0x01;

const UVC_IAD_CLASS: u8 = 0x0E;
const UVC_IAD_SUBCLASS: u8 = 0x03;
const UVC_IAD_PROTOCOL: u8 = 0x00;

const UVC_INTERF_CLASS: u8 = 0x0E;
const UVC_INTERF_SUBCLASS_CONTROL: u8 = 1;
const UVC_INTERF_SUBCLASS_STREAMING: u8 = 2;

const DESC_TYPE_IAD: u8 = 11;

/// Contains information needed to communicate with a UVC device, extracted from the device, configuration, and interface descriptors.
#[derive(Debug)]
pub(crate) struct UvcInfo {
    pub(crate) control_interface: ControlInterface,
    pub(crate) streaming_interfaces: Vec<StreamingInterfaceDesc>,
}

#[derive(Debug)]
pub(crate) struct ControlInterface {
    pub(crate) interface_number: u8,
    /// Interrupt endpoint of the Video Control interface. Optional.
    pub(crate) control_interrupt_ep: Option<u8>,
    pub(crate) topo: Topology,
}

#[derive(Debug, FromBytes)]
#[repr(C)]
#[allow(non_snake_case)]
struct InterfaceAssociationDescriptor {
    bLength: u8,
    bDescriptorType: u8,
    bFirstInterface: u8,
    bInterfaceCount: u8,
    bFunctionClass: u8,
    bFunctionSubClass: u8,
    bFunctionProtocol: u8,
    iFunction: u8,
}

pub(crate) fn detect_uvc(device: &Device<Context>) -> Result<Option<UvcInfo>> {
    // UVC uses an Interface Association Descriptor (IAD) and the corresponding device class.

    let device_desc = device
        .device_descriptor()
        .during(Action::AccessingDeviceDescriptor)?;

    log::trace!(
        "Bus {:03} Device {:03} {:04x}:{:04x}",
        device.bus_number(),
        device.address(),
        device_desc.vendor_id(),
        device_desc.product_id(),
    );

    if device_desc.class_code() != IAD_DEVICE_CLASS
        || device_desc.sub_class_code() != IAD_DEVICE_SUBCLASS
        || device_desc.protocol_code() != IAD_DEVICE_PROTOCOL
    {
        log::trace!("not an IAD device");
        return Ok(None);
    }

    if device_desc.num_configurations() != 1 {
        log::debug!(
            "device has {} configurations, we can only handle 1",
            device_desc.num_configurations()
        );
        return Ok(None);
    }

    let config_desc = device
        .config_descriptor(0)
        .during(Action::AccessingDeviceDescriptor)?;

    let iad = split_descriptors(config_desc.extra()).find_map(|(desc_ty, data)| {
        if desc_ty == DESC_TYPE_IAD {
            match InterfaceAssociationDescriptor::read_from_prefix(data) {
                Some(desc) => Some(desc),
                None => {
                    log::warn!("failed to parse IAD from {:x?}", data);
                    None
                }
            }
        } else {
            None
        }
    });

    let iad = match iad {
        Some(iad) => iad,
        None => {
            log::warn!("found no IAD despite device class indicating that there is one");
            return Ok(None);
        }
    };

    log::debug!("{:?}", iad);

    if iad.bFunctionClass != UVC_IAD_CLASS
        || iad.bFunctionSubClass != UVC_IAD_SUBCLASS
        || iad.bFunctionProtocol != UVC_IAD_PROTOCOL
    {
        log::trace!("not a video device");
        return Ok(None);
    }

    let first_interface = iad.bFirstInterface;
    let last_interface = first_interface + iad.bInterfaceCount - 1;
    let mut control_interface = None;
    let mut streaming_interfaces = Vec::new();
    for interface in config_desc.interfaces() {
        if interface.number() >= first_interface && interface.number() <= last_interface {
            // FIXME: alt setting handling is questionable
            let desc = interface
                .descriptors()
                .next()
                .expect("interface with no descriptors");
            if desc.class_code() != UVC_INTERF_CLASS {
                return err(
                    format!("interface uses unexpected class code {}", desc.class_code()),
                    Action::AccessingDeviceDescriptor,
                );
            }

            match desc.sub_class_code() {
                UVC_INTERF_SUBCLASS_CONTROL => {
                    if control_interface.is_some() {
                        return err(
                            format!("device lists more than one control interface"),
                            Action::AccessingDeviceDescriptor,
                        );
                    }

                    if desc.num_endpoints() > 1 {
                        return err(
                            format!(
                                "control interface has {} endpoints, only 1 is allowed",
                                desc.num_endpoints()
                            ),
                            Action::AccessingDeviceDescriptor,
                        );
                    }

                    let interrupt_ep = match desc.endpoint_descriptors().next() {
                        Some(ep) => {
                            if ep.transfer_type() != TransferType::Interrupt {
                                return err(
                                    format!("control interface has {:?} endpoint, only interrupt EPs are allowed", ep.transfer_type()),
                                    Action::AccessingDeviceDescriptor,
                                );
                            }

                            Some(ep.address())
                        }
                        None => None,
                    };

                    let topo = topo::parse::parse_control_desc(&desc)?;

                    control_interface = Some(ControlInterface {
                        interface_number: desc.interface_number(),
                        control_interrupt_ep: interrupt_ep,
                        topo,
                    });
                }
                UVC_INTERF_SUBCLASS_STREAMING => {
                    streaming_interfaces.push(topo::parse::parse_streaming_descriptor(&desc)?);
                }
                e => {
                    log::warn!(
                        "interface {} uses unexpected subclass code {}, ignoring it",
                        interface.number(),
                        e
                    );
                }
            }
        }
    }

    let control_interface = match control_interface {
        Some(intf) => intf,
        None => {
            return err(
                format!("device does not have a UVC control interface"),
                Action::AccessingDeviceDescriptor,
            )
        }
    };

    Ok(Some(UvcInfo {
        control_interface,
        streaming_interfaces,
    }))
}
