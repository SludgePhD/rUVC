use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let dev = match ruvc::list()?.next() {
        Some(desc) => desc.open()?,
        None => {
            eprintln!("no UVC devices found");
            return Ok(());
        }
    };

    let interface = &dev.streaming_interfaces()[0];

    let interface_id = interface.id();
    let format = interface.formats()[0].index();
    let frame = interface.frames()[1].index();
    let mut interface = dev.streaming_interface_by_id(interface_id);
    let mut stream = interface.start_stream(format, frame)?;

    println!("stream started");

    let mut buf = vec![0; 1024];
    loop {
        stream.read(&mut buf)?;
    }
}
