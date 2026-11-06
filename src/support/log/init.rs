use std::io::{IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

static ARM: Once = Once::new();

pub const Z: &str = "\x1b[0m";

const ERASE: &str = "\r\x1b[2K";

static LIVE: AtomicBool = AtomicBool::new(false);

pub fn arm() {
    ARM.call_once(enable_colour);
}

#[cfg(windows)]
fn enable_colour() {
    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(which: u32) -> *mut core::ffi::c_void;
        fn GetConsoleMode(handle: *mut core::ffi::c_void, mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: *mut core::ffi::c_void, mode: u32) -> i32;
    }

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle.is_null() {
            return;
        }
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) == 0 {
            return;
        }
        let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
    }
}

#[cfg(not(windows))]
fn enable_colour() {}

fn erase() {
    print!("{}", ERASE);
}

fn live_shown() -> bool {
    LIVE.load(Ordering::Relaxed) && std::io::stdout().is_terminal()
}

pub fn emit(color: &str, msg: &str) {
    if live_shown() {
        erase();
        LIVE.store(false, Ordering::Relaxed);
    }
    println!("{}{}{}", color, msg, Z);
    let _ = std::io::stdout().flush();
}

pub fn live(color: &str, msg: &str) {
    if std::io::stdout().is_terminal() {
        if LIVE.load(Ordering::Relaxed) {
            erase();
        }
        print!("{}{}{}", color, msg, Z);
        let _ = std::io::stdout().flush();
        LIVE.store(true, Ordering::Relaxed);
    } else {
        println!("{}{}{}", color, msg, Z);
        let _ = std::io::stdout().flush();
    }
}

#[allow(dead_code)]
pub fn finish_live() {
    if live_shown() {
        LIVE.store(false, Ordering::Relaxed);
        println!();
        let _ = std::io::stdout().flush();
    }
}

pub fn clear_live() {
    if live_shown() {
        erase();
        LIVE.store(false, Ordering::Relaxed);
        let _ = std::io::stdout().flush();
    }
}

pub fn newline() {
    if live_shown() {
        LIVE.store(false, Ordering::Relaxed);
    }
    println!();
    let _ = std::io::stdout().flush();
}

static ECHO_OFF: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
pub fn mute_echo() {
    ECHO_OFF.store(true, Ordering::Relaxed);
}

pub fn restore_echo() {
    if !ECHO_OFF.swap(false, Ordering::Relaxed) {
        return;
    }
    #[cfg(unix)]
    {
        let _ = std::process::Command::new("stty")
            .arg("echo")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

pub fn blank() {
    if live_shown() {
        erase();
        LIVE.store(false, Ordering::Relaxed);
    }
    println!();
    let _ = std::io::stdout().flush();
}
