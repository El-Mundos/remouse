use nix::fcntl::{FcntlArg, OFlag, fcntl};
use nix::libc;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

// Structure for the timestamps within the input events
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TimeVal {
    tv_sec: i64,
    tv_usec: i64,
}

// Structure for the input events from the device
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct InputEvent {
    time: TimeVal,
    event_type: u16,
    code: u16,
    value: i32,
}

// The state of the debounced key
// last_is_released: Whether the last event was a release (true) or anything else (false)
// last_time: The timestamp of the last event (in milliseconds since epoch)
// sent_press: Whether a press event has been sent currently
// sent_release: Whether a release event has been sent yet for the last press event
struct ButtonState {
    last_is_released: bool,
    last_time: i64,
    sent_press: bool,
    sent_release: bool,
}

impl ButtonState {
    fn new() -> Self {
        ButtonState {
            last_is_released: true,
            last_time: 0,
            sent_press: false,
            sent_release: true,
        }
    }
}

// EVIOCGRAB ioctl code
const EVIOCGRAB: libc::c_ulong = 0x40044590; // From linux/input.h

// Debounce constants
// TODO: Read from args/config file
const DEBOUNCE_MS: i64 = 50;
const BTN_SIDE: u16 = 0x113;

fn current_time_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Welcome to the future! (change your clock to past 1970 please)")
        .as_millis() as i64
}

fn open_device(path: &str) -> std::io::Result<File> {
    File::open(path)
}

fn read_event(file: &mut File) -> std::io::Result<Option<InputEvent>> {
    // Initialize an empty inputEvent struct to read into
    let mut event = InputEvent {
        time: TimeVal {
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
            std::mem::size_of::<InputEvent>(), // Specify the size of the byte slice
        )
    };

    // Fill the inputEvent struct with data read from the device
    match file.read_exact(bytes) {
        Ok(_) => Ok(Some(event)),
        Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(e),
    }
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

fn set_nonblocking(file: &File) -> std::io::Result<()> {
    fcntl(file, FcntlArg::F_SETFL(OFlag::O_NONBLOCK))
        .map_err(|e| std::io::Error::from_raw_os_error(e as i32))?;
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

    // Grab device without blocking execution on read_event()
    grab_device(&file)?;
    set_nonblocking(&file)?;

    println!("Device grabbed exclusively!");
    println!("Reading events from {}", device_path);
    println!("Press Ctrl+C to exit");

    loop {
        match read_event(&mut file)? {
            Some(event) => {
                // TODO: Actually process the event and implement debounce logic
                println!(
                    "Got event! Type: {} code: {} value: {}",
                    event.event_type, event.code, event.value
                );
            }
            None => {
                // TODO: Check for silence timeout
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
        // waiting
    }
}
