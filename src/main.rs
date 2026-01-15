use std::env;
use std::fs::{OpenOptions, read_dir};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
    unsafe { tcsetattr(0, TCSANOW, tio); }
}

fn read_key() -> u8 {
    let mut b = [0u8; 1];
    if std::io::stdin().read_exact(&mut b).is_ok() { b[0] } else { 0 }
}

fn clear(out: &mut std::io::StdoutLock) {
    let _ = out.write_all(b"\x1b[2J\x1b[H\x1b[?7l");
}

fn enable_wrap(out: &mut std::io::StdoutLock) {
    let _ = out.write_all(b"\x1b[?7h");
}

fn list_devices() -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(rd) = read_dir("/dev") {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if n.starts_with("cu.") && !n.contains("Bluetooth") {
                v.push(format!("/dev/{}", n));
            }
        }
    }
    v.sort();
    v
}

fn menu_devices(title: &str) -> String {
    let old = set_stdin_raw();
    let mut sel = 0usize;

    loop {
        let items = list_devices();
        if items.is_empty() {
            let mut out = std::io::stdout().lock();
            clear(&mut out);
            let _ = out.write_all(b"No devices found\r\n");
            let _ = out.flush();
            thread::sleep(Duration::from_millis(500));
            continue;
        }
        if sel >= items.len() { sel = items.len() - 1; }

        let mut out = std::io::stdout().lock();
        clear(&mut out);
        let _ = out.write_all(title.as_bytes());
        let _ = out.write_all(b"\r\n--------------------\r\n");

        for (i, it) in items.iter().enumerate() {
            if i == sel { let _ = out.write_all(b"> "); }
            else { let _ = out.write_all(b"  "); }
            let _ = out.write_all(it.as_bytes());
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
            process::exit(0);
        }
    }
}

fn menu_static(title: &str, items: &[String]) -> String {
    let old = set_stdin_raw();
    let mut sel = 0usize;

    loop {
        let mut out = std::io::stdout().lock();
        clear(&mut out);
        let _ = out.write_all(title.as_bytes());
        let _ = out.write_all(b"\r\n--------------------\r\n");

        for (i, it) in items.iter().enumerate() {
            if i == sel { let _ = out.write_all(b"> "); }
            else { let _ = out.write_all(b"  "); }
            let _ = out.write_all(it.as_bytes());
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
            process::exit(0);
        }
    }
}

fn baud_to_flag(s: &str) -> speed_t {
    match s {
        "9600" => B9600,
        "19200" => B19200,
        "38400" => B38400,
        "57600" => B57600,
        "115200" => B115200,
        _ => panic!("invalid baud"),
    }
}

fn timestamp() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let ms = d.as_millis() % 1000;
    let s = d.as_secs() % 60;
    let m = (d.as_secs() / 60) % 60;
    let h = (d.as_secs() / 3600) % 24;
    format!("[{:02}:{:02}:{:02}.{:03}] ", h, m, s, ms)
}

fn draw_status(dev: &str, baud: &str, raw: bool, rx: u64, tx: u64) {
    let mode = if raw { "RAW" } else { "LINE" };
    let line = format!(
        "DEV: {} | BAUD: {} | MODE: {} | RX: {} | TX: {}",
        dev, baud, mode, rx, tx
    );
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(b"\x1b[s\x1b[999;1H\x1b[2K");
    let _ = out.write_all(line.as_bytes());
    let _ = out.write_all(b"\x1b[u");
    let _ = out.flush();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let (device, baud_str) = if args.len() >= 3 {
        (args[1].clone(), args[2].clone())
    } else {
        let d = menu_devices("Select serial device");
        let b = menu_static("Select baud rate", &vec![
            "9600".into(),"19200".into(),"38400".into(),"57600".into(),"115200".into()
        ]);
        (d, b)
    };

    let baud = baud_to_flag(&baud_str);
    let mut port = OpenOptions::new().read(true).write(true).open(&device).unwrap();
    set_raw(port.as_raw_fd(), baud);

    let rx_count = Arc::new(AtomicU64::new(0));
    let tx_count = Arc::new(AtomicU64::new(0));

    let mut rx = port.try_clone().unwrap();
    let rx_c = rx_count.clone();
    let dev = device.clone();
    let baud_s = baud_str.clone();

    thread::spawn(move || {
        let mut buf = [0u8; 256];
        let mut out = std::io::stdout().lock();
        let mut new_line = true;
        loop {
            match rx.read(&mut buf) {
                Ok(n) if n > 0 => {
                    rx_c.fetch_add(n as u64, Ordering::Relaxed);
                    for &b in &buf[..n] {
                        if new_line {
                            let _ = out.write_all(timestamp().as_bytes());
                            new_line = false;
                        }
                        let _ = out.write_all(&[b]);
                        if b == b'\n' { new_line = true; }
                    }
                    let _ = out.flush();
                    draw_status(&dev, &baud_s, false, rx_c.load(Ordering::Relaxed), 0);
                }
                _ => thread::sleep(Duration::from_millis(2)),
            }
        }
    });

    let mut raw = false;
    let mut byte = [0u8; 1];

    loop {
        if raw {
            if std::io::stdin().read_exact(&mut byte).is_err() { break; }
            if byte[0] == 0x03 || byte[0] == 0x04 { break; }
            if byte[0] == 0x12 { raw = false; continue; }
            let _ = port.write_all(&byte);
            tx_count.fetch_add(1, Ordering::Relaxed);
        } else {
            let mut line = String::new();
            match std::io::stdin().read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if line.as_bytes().contains(&0x12) {
                        raw = true;
                        continue;
                    }
                    let _ = port.write_all(line.as_bytes());
                    tx_count.fetch_add(line.len() as u64, Ordering::Relaxed);
                }
                Err(_) => break,
            }
        }
        draw_status(&device, &baud_str, raw, rx_count.load(Ordering::Relaxed), tx_count.load(Ordering::Relaxed));
    }
}