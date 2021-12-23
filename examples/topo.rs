use ruvc::UvcDeviceDesc;

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

    println!("{:#?}", dev.topology());

    Ok(())
}
