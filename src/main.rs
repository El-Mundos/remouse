use std::fs::File;
use std::io::Read;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct timeVal {
    tv_sec: i64,
    tv_usec: i64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct inputEvent {
    time: timeVal,
    event_type: u16,
    code: u16,
    value: i32,
}

fn open_device(path: &str) -> std::io::Result<File> {
    File::open(path)
}

fn read_event(file: &mut File) -> std::io::Result<inputEvent> {
    let mut event = inputEvent {
        time: timeVal {
            tv_sec: 0,
            tv_usec: 0,
        },
        event_type: 0,
        code: 0,
        value: 0,
    };

    let bytes = unsafe {
        std::slice::from_raw_parts_mut(
            &mut event as *mut _ as *mut u8,
            std::mem::size_of::<inputEvent>(),
        )
    };

    file.read_exact(bytes)?;

    Ok(event)
}

fn main() -> std::io::Result<()> {
    // Get device path from args
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <device_path>", args[0]);
        std::process::exit(1);
    }

    let device_path = &args[1];

    // Open device
    let mut file = open_device(device_path)?;

    println!("Reading events from {}", device_path);
    println!("Press Ctrl+C to exit");

    loop {
        let event = read_event(&mut file)?;

        // Print the event
        println!(
            "type: {} code: {} value: {}",
            event.event_type, event.code, event.value
        );
    }
}
