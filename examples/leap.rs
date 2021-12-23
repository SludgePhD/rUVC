use std::io::Read;

use ruvc::{
    camera::*,
    control::ProbeHint,
    processing_unit::*,
    streaming_interface::{Commit, Probe},
    UvcDeviceDesc,
};

const LEAP_VID: u16 = 0xf182;
const LEAP_PID: u16 = 0x0003;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    for desc in ruvc::list()? {
        if desc.vendor_id() == LEAP_VID && desc.product_id() == LEAP_PID {
            go(desc)?;
            return Ok(());
        }
    }

    eprintln!("no matching device found");
    Ok(())
}

fn go(desc: UvcDeviceDesc) -> Result<(), Box<dyn std::error::Error>> {
    let dev = desc.open()?;
    println!("opened device '{}'", dev.read_product_string()?);

    let desc = match dev
        .topology()
        .units()
        .iter()
        .find_map(|unit| unit.as_processing_unit())
    {
        Some(proc) => proc,
        None => {
            eprintln!("couldn't find any processing units");
            return Ok(());
        }
    };

    let camera_id = match dev
        .topology()
        .inputs()
        .iter()
        .find_map(|input| input.as_camera_id())
    {
        Some(id) => id,
        None => {
            eprintln!("couldn't find a camera input");
            return Ok(());
        }
    };

    let id = desc.id();
    let mut pu = dev.processing_unit_by_id(id);
    let mut cam = dev.camera_terminal_by_id(camera_id);

    // read opaque calibration data
    let mut calibration = Vec::new();
    for addr in 0..256 {
        pu.set_control::<Sharpness>(addr)?;
        calibration.push(pu.read_control::<Saturation>()?);
    }
    println!("calibration data: {:x?}", calibration);

    // init block
    pu.set_control::<WhiteBalanceTemperature>(127)?;
    cam.set_control::<FocusAbs>(1000)?;
    pu.set_control::<Contrast>(1)?;
    pu.set_control::<Brightness>(4)?;
    cam.set_control::<FocusAbs>(1000)?;
    cam.set_control::<ZoomAbs>(200)?;
    pu.set_control::<Gain>(16)?;
    pu.set_control::<Gamma>(1)?;
    // configure HDR/LEDs/etc
    pu.set_control::<Contrast>(0)?;
    pu.set_control::<Contrast>(0b01000_100)?;
    pu.set_control::<Contrast>(0b01000_010)?;
    pu.set_control::<Contrast>(0b01000_011)?;
    pu.set_control::<Contrast>(0x0006)?;
    pu.set_control::<Contrast>(0x3C05)?;
    pu.set_control::<WhiteBalanceTemperature>(127)?;

    println!("setup complete");

    let interface = &dev.streaming_interfaces()[0];
    let mut st = dev.streaming_interface_by_id(interface.id());
    let mut params = st.read_control_max::<Probe>()?;
    log::trace!("GET_MAX(PROBE) = {:?}", params);

    params.bmHint = params.bmHint | ProbeHint::FIX_FRAME_INTERVAL;
    params.dwFrameInterval = 86956;
    log::trace!("SET_CUR(PROBE) = {:?}", params);
    st.set_control::<Probe>(params)?;

    params = st.read_control::<Probe>()?;
    log::trace!("GET_CUR(PROBE) = {:?}", params);
    st.set_control::<Commit>(params)?;
    let mut stream = st.start_stream_no_negotiate();

    println!("stream started");

    let mut buf = vec![0; params.dwMaxPayloadTransferSize as usize];
    loop {
        stream.read(&mut buf)?;
    }
}
