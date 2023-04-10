# serial_write

Simplifying serial output in a `no_std` environment, both string and numeric.

## List of possible output types

- `&str`
- `i8`, `i16`, `i32`, `i64`, `isize`
- `u8`, `u16`, `u32`, `u64`, `usize`
- `f32`, `f64`
- `&[i8]`, `&[i16]`, `&[i32]`, `&[i64]`, `&[isize]`
- `&[u8]`, `&[u16]`, `&[u32]`, `&[u64]`, `&[usize]`
- `&[f32]`, `&[f64]`

## How to use

### 1. Prepare `SerialPort`

```rust
// Set the USB bus
let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(...));

// Set the serial port
let mut serial = SerialPort::new(&usb_bus);
```

### 2. Create `Writer`

```rust
let mut writer = Writer::new();
```

### 3. Output strings or numbers.

```rust
// Output "123".
writer.write_usize(123, &mut serial);

// Output "123" and break line.
writer.writeln_usize(123, &mut serial);

// Output to 2 decimal places ("12.34").
writer.write_usize(12.3456, 2, &mut serial);
```