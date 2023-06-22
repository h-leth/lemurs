//! Adapted From https://github.com/jonay2000/chvt-rs

use libc::{c_int, c_ulong};
use nix::errno::Errno;
use nix::fcntl::{self, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::close;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

const VT_ACTIVATE: c_ulong = 0x5606;
const VT_WAITACTIVE: c_ulong = 0x5607;

// Request Number to get Keyboard Type
const KDGKBTYPE: c_ulong = 0x4B33;

const KB_101: u8 = 0x02;
const KB_84: u8 = 0x01;

#[derive(Debug)]
pub enum ChvtError {
    Activate(i32),
    WaitActive(i32),
    Close,
    OpenConsole,
    NotAConsole,
    GetFD,
}

impl Error for ChvtError {}
impl Display for ChvtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <dyn Debug>::fmt(self, f)
    }
}

fn is_a_console(fd: c_int) -> bool {
    let mut arg = 0;
    if unsafe { libc::ioctl(fd, KDGKBTYPE, &mut arg) } > 0 {
        return false;
    }

    (arg == KB_101) || (arg == KB_84)
}

fn open_a_console(filename: &str) -> Result<c_int, ChvtError> {
    for oflag in [OFlag::O_RDWR, OFlag::O_RDONLY, OFlag::O_WRONLY] {
        match fcntl::open(filename, oflag, Mode::empty()) {
            Ok(fd) => {
                if !is_a_console(fd) {
                    close(fd).map_err(|_| ChvtError::Close)?;
                    return Err(ChvtError::NotAConsole);
                }

                return Ok(fd);
            }
            Err(Errno::EACCES) => continue,
            _ => break,
        }
    }

    Err(ChvtError::OpenConsole)
}

fn get_fd() -> Result<c_int, ChvtError> {
    if let Ok(fd) = open_a_console("/dev/tty") {
        return Ok(fd);
    }

    if let Ok(fd) = open_a_console("/dev/tty") {
        return Ok(fd);
    }

    if let Ok(fd) = open_a_console("/dev/tty0") {
        return Ok(fd);
    }

    if let Ok(fd) = open_a_console("/dev/vc/0") {
        return Ok(fd);
    }

    if let Ok(fd) = open_a_console("/dev/console") {
        return Ok(fd);
    }

    for fd in 0..3 {
        if is_a_console(fd) {
            return Ok(fd);
        }
    }

    // If all attempts fail Error
    Err(ChvtError::GetFD)
}

pub unsafe fn chvt(ttynum: i32) -> Result<(), ChvtError> {
    let fd = get_fd()?;

    let activate = unsafe { libc::ioctl(fd, VT_ACTIVATE, ttynum as c_int) };
    if activate > 0 {
        return Err(ChvtError::Activate(activate));
    }

    let wait = unsafe { libc::ioctl(fd, VT_WAITACTIVE, ttynum) };
    if wait > 0 {
        return Err(ChvtError::WaitActive(wait));
    }

    close(fd).map_err(|_| ChvtError::Close)?;

    Ok(())
}
