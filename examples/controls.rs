use std::{
    any::type_name,
    fmt::{Debug, Display},
};

use ruvc::{
    camera::*,
    processing_unit::*,
    topo::{
        CameraControls, CameraId, CameraTerminalDesc, InputTerminalKind, ProcessingUnitControls,
        ProcessingUnitDesc, SelectorUnitDesc, UnitKind,
    },
    UvcDevice, UvcDeviceDesc,
};

fn main() -> ruvc::Result<()> {
    env_logger::init();

    for desc in ruvc::list()? {
        match list_device(desc) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("error: {}", e);
                eprintln!("(skipping device)");
            }
        }
    }

    Ok(())
}

fn list_device(desc: UvcDeviceDesc) -> ruvc::Result<()> {
    print!("{:04x}:{:04x} ", desc.vendor_id(), desc.product_id());

    let dev = desc.open()?;
    println!(
        "[{}] {}",
        dev.read_manufacturer_string()?,
        dev.read_product_string()?,
    );

    for input in dev.topology().inputs() {
        match input.terminal_kind() {
            InputTerminalKind::Camera(cam) => {
                let id = input.as_camera_id().unwrap(); // FIXME bad API
                list_camera_controls(&dev, id, cam)?
            }
            _ => {}
        }
    }

    for unit in dev.topology().units() {
        match unit.unit_kind() {
            UnitKind::Selector(desc) => list_selector_unit_controls(&dev, desc)?,
            UnitKind::Processing(desc) => list_processing_unit_controls(&dev, desc)?,
            _ => {}
        }
    }

    // TODO outputs?

    Ok(())
}

fn list_camera_controls(
    dev: &UvcDevice,
    id: CameraId,
    desc: &CameraTerminalDesc,
) -> ruvc::Result<()> {
    println!("Camera Terminal controls ({:?}):", id);

    let cam = dev.camera_terminal_by_id(id);
    let c = desc.controls();
    if c.contains(CameraControls::SCANNING_MODE) {
        print_cam_control::<ScanningMode>(&cam)?;
    }
    if c.contains(CameraControls::AUTO_EXPOSURE_MODE) {
        print_cam_control::<AutoExposureMode>(&cam)?;
    }
    if c.contains(CameraControls::AUTO_EXPOSURE_PRIORITY) {
        print_cam_control::<AutoExposurePriority>(&cam)?;
    }
    if c.contains(CameraControls::EXPOSURE_TIME_ABS) {
        print_cam_control::<ExposureTimeAbs>(&cam)?;
    }
    if c.contains(CameraControls::EXPOSURE_TIME_REL) {
        print_cam_control::<ExposureTimeRel>(&cam)?;
    }
    if c.contains(CameraControls::FOCUS_ABS) {
        print_cam_control::<FocusAbs>(&cam)?;
    }
    if c.contains(CameraControls::FOCUS_REL) {
        print_cam_control::<FocusRel>(&cam)?;
    }
    if c.contains(CameraControls::IRIS_ABS) {
        print_cam_control::<IrisAbs>(&cam)?;
    }
    if c.contains(CameraControls::IRIS_REL) {
        print_cam_control::<IrisRel>(&cam)?;
    }
    if c.contains(CameraControls::ZOOM_ABS) {
        print_cam_control::<ZoomAbs>(&cam)?;
    }
    if c.contains(CameraControls::FOCUS_AUTO) {
        print_cam_control::<FocusAuto>(&cam)?;
    }
    if c.contains(CameraControls::FOCUS_SIMPLE) {
        print_cam_control::<FocusSimple>(&cam)?;
    }
    // TODO complete

    Ok(())
}

fn print_cam_control<C: CameraControl>(cam: &CameraTerminal<'_>) -> ruvc::Result<()>
where
    C::Value: Debug,
{
    let name = type_name::<C>().split("::").last().unwrap();
    println!(
        "- {}: {:?} ({:?}-{:?}, step {:?}, default {:?})",
        name,
        cam.read_control::<C>()?,
        cam.read_control_min::<C>()?,
        cam.read_control_max::<C>()?,
        cam.read_control_res::<C>()?,
        cam.read_control_default::<C>()?,
    );
    Ok(())
}

fn list_selector_unit_controls(dev: &UvcDevice, desc: &SelectorUnitDesc) -> ruvc::Result<()> {
    eprintln!("NYI: selector unit controls");
    Ok(())
}

fn list_processing_unit_controls(dev: &UvcDevice, desc: &ProcessingUnitDesc) -> ruvc::Result<()> {
    println!("Processing Unit controls ({:?}):", desc.id());

    let pu = dev.processing_unit_by_id(desc.id());
    let c = desc.controls();
    if c.contains(ProcessingUnitControls::BRIGHTNESS) {
        print_pu_control::<Brightness>(&pu)?;
    }
    if c.contains(ProcessingUnitControls::CONTRAST) {
        print_pu_control::<Contrast>(&pu)?;
    }
    if c.contains(ProcessingUnitControls::HUE) {
        print_pu_control::<Hue>(&pu)?;
    }
    if c.contains(ProcessingUnitControls::SATURATION) {
        print_pu_control::<Saturation>(&pu)?;
    }
    if c.contains(ProcessingUnitControls::SHARPNESS) {
        print_pu_control::<Sharpness>(&pu)?;
    }
    if c.contains(ProcessingUnitControls::GAMMA) {
        print_pu_control::<Gamma>(&pu)?;
    }
    if c.contains(ProcessingUnitControls::WHITE_BALANCE_TEMPERATURE) {
        print_pu_control::<WhiteBalanceTemperature>(&pu)?;
    }
    if c.contains(ProcessingUnitControls::WHITE_BALANCE_COMPONENT) {
        print_pu_control::<WhiteBalanceComponent>(&pu)?;
    }
    if c.contains(ProcessingUnitControls::BACKLIGHT_COMPENSATION) {
        print_pu_control::<BacklightCompensation>(&pu)?;
    }
    if c.contains(ProcessingUnitControls::GAIN) {
        print_pu_control::<Gain>(&pu)?;
    }
    if c.contains(ProcessingUnitControls::POWER_LINE_FREQUENCY) {
        print_pu_control::<PowerLineFrequency>(&pu)?;
    }
    // TODO

    Ok(())
}

fn print_pu_control<C: ProcessingUnitControl>(pu: &ProcessingUnit<'_>) -> ruvc::Result<()>
where
    C::Value: Debug,
{
    let name = type_name::<C>().split("::").last().unwrap();
    println!(
        "- {}: {:?} ({:?}-{:?}, step {:?}, default {:?})",
        name,
        pu.read_control::<C>()?,
        pu.read_control_min::<C>()?,
        pu.read_control_max::<C>()?,
        pu.read_control_res::<C>()?,
        pu.read_control_default::<C>()?,
    );
    Ok(())
}
