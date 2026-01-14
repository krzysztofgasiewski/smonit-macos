use std::env;
use std::fs::{OpenOptions, read_dir};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::thread;
use std::time::Duration;
use std::process;

use libc::*;

fn set_raw(fd: i32, baud: speed_t) {
    unsafe {
        let mut tio: termios = std::mem::zeroed();
        tcgetattr(fd, &mut tio);
        cfmakeraw(&mut tio);
        cfsetspeed(&mut tio, baud);
        tio.c_cflag |= CLOCAL | CREAD;
        tio.c_cflag &= !CSTOPB;
        tio.c_cflag &= !PARENB;
        tio.c_cflag &= !CSIZE;
        tio.c_cflag |= CS8;
        tcsetattr(fd, TCSANOW, &tio);
    }
}

fn set_stdin_raw() -> termios {
    unsafe {
        let mut tio: termios = std::mem::zeroed();
        tcgetattr(0, &mut tio);
        let old = tio;
        cfmakeraw(&mut tio);
        tcsetattr(0, TCSANOW, &tio);
        old
    }
}

fn restore_stdin(tio: &termios) {
    unsafe {
        tcsetattr(0, TCSANOW, tio);
    }
}

fn read_key() -> u8 {
    let mut b = [0u8; 1];
    std::io::stdin().read_exact(&mut b).unwrap();
    b[0]
}

fn clear(out: &mut std::io::StdoutLock) {
    let _ = out.write_all(b"\x1b[2J\x1b[H\x1b[?7l");
}

fn enable_wrap(out: &mut std::io::StdoutLock) {
    let _ = out.write_all(b"\x1b[?7h");
}

fn menu(title: &str, items: &[String]) -> String {
    let old = set_stdin_raw();
    let mut sel = 0usize;

    loop {
        let mut out = std::io::stdout().lock();
        clear(&mut out);

        let _ = out.write_all(title.as_bytes());
        let _ = out.write_all(b"\r\n");
        let _ = out.write_all(b"--------------------\r\n");

        for (i, item) in items.iter().enumerate() {
            if i == sel {
                let _ = out.write_all(b"> ");
            } else {
                let _ = out.write_all(b"  ");
            }
            let _ = out.write_all(item.as_bytes());
            let _ = out.write_all(b"\r\n");
        }

        let _ = out.flush();

        let k = read_key();
        if k == 0x1b {
            let _ = read_key();
            match read_key() {
                b'A' => if sel > 0 { sel -= 1; }
                b'B' => if sel + 1 < items.len() { sel += 1; }
                _ => {}
            }
        } else if k == b'\n' || k == b'\r' {
            enable_wrap(&mut out);
            restore_stdin(&old);
            let _ = out.write_all(b"\x1b[2J\x1b[H");
            let _ = out.flush();
            return items[sel].clone();
        } else if k == 0x03 {
            enable_wrap(&mut out);
            restore_stdin(&old);
            let _ = out.write_all(b"\x1b[2J\x1b[H");
            let _ = out.flush();
            process::exit(0);
        }
    }
}

fn list_devices() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(rd) = read_dir("/dev") {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("cu.") {
                out.push(format!("/dev/{}", name));
            }
        }
    }
    out.sort();
    out
}

fn baud_to_flag(s: &str) -> speed_t {
    match s {
        "9600" => B9600,
        "19200" => B19200,
        "38400" => B38400,
        "57600" => B57600,
        "115200" => B115200,
        _ => B115200,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let (device, baud_str) = if args.len() >= 3 {
        (args[1].clone(), args[2].clone())
    } else {
        let devs = list_devices();
        if devs.is_empty() {
            eprintln!("No serial devices found");
            return;
        }
        let d = menu("Select serial device", &devs);
        let bauds = vec![
            "9600".to_string(),
            "19200".to_string(),
            "38400".to_string(),
            "57600".to_string(),
            "115200".to_string(),
        ];
        let b = menu("Select baud rate", &bauds);
        (d, b)
    };

    let baud = baud_to_flag(&baud_str);

    let mut port = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&device)
        .expect("Failed to open serial device");

    let fd = port.as_raw_fd();
    set_raw(fd, baud);

    let mut read_port = port.try_clone().unwrap();

    thread::spawn(move || {
        let mut buf = [0u8; 256];
        let mut out = std::io::stdout().lock();
        loop {
            match read_port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let _ = out.write_all(&buf[..n]);
                    let _ = out.flush();
                }
                _ => thread::sleep(Duration::from_millis(5)),
            }
        }
    });

    let stdin = std::io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        if stdin.read_line(&mut input).is_ok() {
            let _ = port.write_all(input.as_bytes());
        }
    }
}
