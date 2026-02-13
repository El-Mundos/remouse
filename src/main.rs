use std::fs::File;
use std::io::Read;
use std::os::unix::io::AsRawFd;

// Structure for the timestamps within the input events
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct timeVal {
    tv_sec: i64,
    tv_usec: i64,
}

// Structure for the input events from the device
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct inputEvent {
    time: timeVal,
    event_type: u16,
    code: u16,
    value: i32,
}

// EVIOCGRAB ioctl code
const EVIOCGRAB: libc::c_ulong = 0x40044590; // From linux/input.h

fn open_device(path: &str) -> std::io::Result<File> {
    File::open(path)
}

fn read_event(file: &mut File) -> std::io::Result<inputEvent> {
    // Initialize an empty inputEvent struct to read into
    let mut event = inputEvent {
        time: timeVal {
            tv_sec: 0,
            tv_usec: 0,
        },
        event_type: 0,
        code: 0,
        value: 0,
    };

    // Get a mutable byte slice that points to the inputEvent struct
    let bytes = unsafe {
        std::slice::from_raw_parts_mut(
            &mut event as *mut _ as *mut u8, // Cast the inputEvent struct to a byte pointer
            std::mem::size_of::<inputEvent>(), // Specify the size of the byte slice
        )
    };

    // Fill the inputEvent struct with data read from the device
    file.read_exact(bytes)?;

    Ok(event)
}

// Grab device for exclusive access so other processes can't read from it
fn grab_device(file: &File) -> std::io::Result<()> {
    // Open file descriptor
    let fd = file.as_raw_fd();
    // Use EVIOCGRAB ioctl to grab the device (1 = grab, 0 = release)
    let result = unsafe { libc::ioctl(fd, EVIOCGRAB as libc::c_ulong, 1) };

    if result < 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(())
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

    // Grab device so it's not used elsewhere
    grab_device(&file)?;

    println!("Device grabbed exclusively!");
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
