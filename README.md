# 🔌 smonit — Simple Serial Monitor for macOS

A lightweight, fast, and no-nonsense **serial monitor for macOS (Apple Silicon)** written in Rust.  
`smonit` is designed to be dead simple: pick a serial device, choose a baud rate, and start talking to your hardware.

Perfect for ESP32, Arduino, microcontrollers, dev boards, and general serial debugging — without heavy GUIs or bloated dependencies.

---

## ✨ Features

- 📟 **Interactive TUI menu**
  - Arrow-key navigation
  - Select serial device and baud rate visually
- ⚡ **Zero-config usage**
  - Works out of the box on macOS
- 🔁 **Real-time bidirectional communication**
  - Incoming serial data is printed instantly
  - Anything you type is sent directly to the device
- 🧼 **Raw terminal mode**
  - Clean input/output, no weird buffering
- 🍎 **macOS-friendly**
  - Automatically detects `/dev/cu.*` devices
- 🧠 **Minimal & transparent**
  - No magic, no hidden behavior

---

## 🧰 Installation

### Download prebuilt binary (recommended)

1. Download the `smonit` binary
2. Install it system-wide:

```bash
sudo mv smonit /usr/local/bin
```

Now you can run `smonit` from anywhere in your terminal 🎉

---

## 🚀 Usage

### Interactive mode (no arguments)

```bash
smonit
```

You will be prompted to:
1. Select a serial device
2. Select a baud rate

Controls:
- ⬆️ / ⬇️ — navigate
- ⏎ Enter — confirm
- ⌃ Ctrl+C — exit

---

### Direct mode (CLI arguments)

```bash
smonit /dev/cu.usbserial-0001 115200
```

Format:
```bash
smonit <device> <baud>
```

---

## ⌨️ Runtime Behavior

- Anything you type is sent directly to the serial device
- Incoming serial data is printed instantly
- Works great with Arduino, ESP32, and CLI firmware

Exit anytime with **Ctrl+C**

---

## 📦 Supported Baud Rates

- 9600
- 19200
- 38400
- 57600
- 115200

---

## 🍎 Platform Support

| Platform | Status |
|---------|--------|
| macOS (Apple Silicon) | ✅ Supported |
| macOS (Intel) | ⚠️ Untested |
| Linux | ❌ Not supported |
| Windows | ❌ Not supported |

---

## 🛠 Built With

- 🦀 Rust
- POSIX `termios`
- Zero external runtime dependencies

---

## 💬 Feedback

This is an early release.  
If you find bugs, have ideas, or want features — feedback is very welcome!

---
