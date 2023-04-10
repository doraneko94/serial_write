#![no_std]

use usb_device::class_prelude::{UsbBus, UsbError};
use usbd_serial::SerialPort;
use numtoa::NumToA;

const F32_EXP: [f32; 8] = [
    1f32, 10f32, 100f32, 1_000f32, 10_000f32, 100_000f32, 1_000_000f32, 10_000_000f32
];
const F64_EXP: [f64; 16] = [
    1f64, 10f64, 100f64, 1_000f64, 10_000f64, 100_000f64, 1_000_000f64, 10_000_000f64,
    100_000_000f64, 1_000_000_000f64, 10_000_000_000f64, 100_000_000_000f64, 1_000_000_000_000f64, 
    10_000_000_000_000f64, 100_000_000_000_000f64, 1_000_000_000_000_000f64
];
const U64_BASE: u64 = 10_000_000_000_000_000_000;

/// Output structure containing a buffer for `numtoa` crate.
pub struct Writer {
    buf: [u8; 20]
}

macro_rules! handle_err0 {
    ($f: expr) => {
        match $f {
            Ok(n) => { Ok(n) }
            Err(e) => { return Err((e, 0)); }
        }
    };
}
macro_rules! handle_err1 {
    ($f: expr, $count: ident) => {
        match $f {
            Ok(n) => { $count += n; }
            Err(e) => { return Err((e, $count)); }
        }
    };
}
macro_rules! handle_err2 {
    ($f: expr, $count: ident) => {
        match $f {
            Ok(n) => { $count += n; }
            Err((e, n)) => { return Err((e, $count+n)); }
        }
    };
}

macro_rules! write_int {
    ($(#[$meta: meta])*
    $int: ty, $name: ident) => {
        $(#[$meta])*
        pub fn $name<U: UsbBus>(&mut self, val: $int, serial: &mut SerialPort<U>) -> Result<usize, (UsbError, usize)> {
            handle_err0!(serial.write(val.numtoa(10, &mut self.buf)))
        }
    };
}
macro_rules! write_float {
    ($(#[$meta: meta])*
    $float: ty, $name: ident, $nodp_lim: expr, $exp: ident) => {
        $(#[$meta])*
        pub fn $name<U: UsbBus>(&mut self, val: $float, nodp: usize, serial: &mut SerialPort<U>) -> Result<usize, (UsbError, usize)> {
            let mut f = val;
            let mut count = 0;
            if f < 0.0 {
                f = -f;
                handle_err2!(self.write_str("-", serial), count)
            }
            let mut z_count_base = 0;
            while f > U64_BASE as $float * U64_BASE as $float {
                f /= U64_BASE as $float;
                z_count_base += 1;
            }
            let mut z_count = 0;
            while f > U64_BASE as $float {
                f /= 10.0;
                z_count += 1;
            }
            let int = f as u64;
            handle_err2!(self.write_u64(int, serial), count);
            for _ in 0..z_count_base {
                handle_err2!(self.write_str("0000000000000000000", serial), count);
            }
            for _ in 0..z_count {
                handle_err2!(self.write_str("0", serial), count);
            }
            if nodp == 0 { return Ok(count); }
            let nodp = if nodp > $nodp_lim { $nodp_lim } else { nodp };
            let frac = ((f - int as $float) * $exp[nodp]) as u64;
            handle_err2!(self.write_str(".", serial), count);
            let mut nodp = nodp;
            while nodp > 1 && frac as $float < $exp[nodp - 1] {
                handle_err2!(self.write_str("0", serial), count);
                nodp -= 1;
            }
            handle_err2!(self.write_u64(frac, serial), count);
            Ok(count)
        }
    };
}
macro_rules! write_float_exp {
    ($(#[$meta: meta])*
    $float: ty, $name: ident, $nodp_lim: expr, $exp: ident, $f: ident) => {
        $(#[$meta])*
        pub fn $name<U: UsbBus>(&mut self, val: $float, nodp: usize, serial: &mut SerialPort<U>) -> Result<usize, (UsbError, usize)> {
            let mut f = val;
            let mut count = 0;
            let mut exp: i16 = 0;
            if f < 0.0 {
                f = -f;
                handle_err2!(self.write_str("-", serial), count)
            } else {
                handle_err2!(self.write_str(" ", serial), count)
            }
            while f >= 10.0 {
                exp += 1;
                f /= 10.0;
            }
            while f < 1.0 {
                exp -= 1;
                f *= 10.0;
            }
            handle_err2!(self.$f(f, nodp, serial), count);
            handle_err2!(self.write_str("e", serial), count);
            if exp < 0 {
                handle_err2!(self.write_str("-", serial), count);
                exp = -exp;
                if exp < 10 {
                    handle_err2!(self.write_str("0", serial), count);
                }
            } else {
                if exp < 100 { handle_err2!(self.write_str("0", serial), count); }
                if exp < 10 { handle_err2!(self.write_str("0", serial), count); }
            }
            handle_err2!(self.write_i16(exp, serial), count);
            Ok(count)
        }
    };
}
macro_rules! write_int_slice {
    ($(#[$meta: meta])*
    $int: ty, $name: ident, $f: ident) => {
        $(#[$meta])*
        pub fn $name<U: UsbBus>(&mut self, val: &[$int], serial: &mut SerialPort<U>) -> Result<usize, (UsbError, usize)> {
            let mut count = 0;
            handle_err2!(self.write_str("[ ", serial), count);
            for &val in val.iter() {
                handle_err2!(self.$f(val, serial), count);
                handle_err2!(self.write_str(", ", serial), count);
            }
            handle_err2!(self.write_str("]", serial), count);
            Ok(count)
        }
    };
}
macro_rules! write_float_slice {
    ($(#[$meta: meta])*
    $float: ty, $name: ident, $f: ident) => {
        $(#[$meta])*
        pub fn $name<U: UsbBus>(&mut self, val: &[$float], nodp: usize, serial: &mut SerialPort<U>) -> Result<usize, (UsbError, usize)> {
            let mut count = 0;
            handle_err2!(self.write_str("[ ", serial), count);
            for &val in val.iter() {
                handle_err2!(self.$f(val, nodp, serial), count);
                handle_err2!(self.write_str(", ", serial), count);
            }
            handle_err2!(self.write_str("]", serial), count);
            Ok(count)
        }
    };
}
macro_rules! writeln_str_int {
    ($(#[$meta: meta])*
    $type: ty, $name: ident, $f: ident) => {
        $(#[$meta])*
        pub fn $name<U: UsbBus>(&mut self, val: $type, serial: &mut SerialPort<U>) -> Result<usize, (UsbError, usize)> {
            let mut count = 0;
            handle_err2!(self.$f(val, serial), count);
            handle_err1!(self._writeln(serial), count);
            Ok(count)
        }
    };
}
macro_rules! writeln_float {
    ($(#[$meta: meta])*
    $type: ty, $name: ident, $f: ident) => {
        $(#[$meta])*
        pub fn $name<U: UsbBus>(&mut self, val: $type, nodp: usize, serial: &mut SerialPort<U>) -> Result<usize, (UsbError, usize)> {
            let mut count = 0;
            handle_err2!(self.$f(val, nodp, serial), count);
            handle_err1!(self._writeln(serial), count);
            Ok(count)
        }
    };
}

impl Writer {
    /// Initialize Writer.
    pub fn new() -> Self {
        Self { buf: [0u8; 20] }
    }
    fn _writeln<U: UsbBus>(&self, serial: &mut SerialPort<U>) -> Result<usize, UsbError> {
        serial.write(b"\r\n")
    }
    /// Output `&str`.
    pub fn write_str<U: UsbBus>(&self, str: &str, serial: &mut SerialPort<U>) -> Result<usize, (UsbError, usize)> {
        match serial.write(str.as_bytes()) {
            Ok(n) => Ok(n),
            Err(e) => Err((e, 0))
        }
    }
    write_int!(/// Output `i8`.
        i8, write_i8);
    write_int!(/// Output `i16`.
        i16, write_i16);
    write_int!(/// Output `i32`.
        i32, write_i32);
    write_int!(/// Output `i64`.
        i64, write_i64);
    write_int!(/// Output `isize`.
        isize, write_isize);
    write_int!(/// Output `u8`.
        u8, write_u8);
    write_int!(/// Output `u16`.
        u16, write_u16);
    write_int!(/// Output `u32`.
        u32, write_u32);
    write_int!(/// Output `u64`.
        u64, write_u64);
    write_int!(/// Output `usize`.
        usize, write_usize);
    write_float!(/// Output `f32` to `nodp` decimal places.
        f32, write_f32, 7, F32_EXP);
    write_float!(/// Output `f64` to `nodp` decimal places.
        f64, write_f64, 15, F64_EXP);
    write_float_exp!(/// Output `f32` to `nodp` decimal places in exponential format.
        f32, write_f32_exp, 7, F32_EXP, write_f32);
    write_float_exp!(/// Output `f64` to `nodp` decimal places in exponential format.
        f64, write_f64_exp, 15, F64_EXP, write_f64);
    write_int_slice!(/// Output `&[i8]`.
        i8, write_i8_slice, write_i8);
    write_int_slice!(/// Output `&[i16]`.
        i16, write_i16_slice, write_i16);
    write_int_slice!(/// Output `&[i32]`.
        i32, write_i32_slice, write_i32);
    write_int_slice!(/// Output `&[i64]`.
        i64, write_i64_slice, write_i64);
    write_int_slice!(/// Output `&[isize]`.
        isize, write_isize_slice, write_isize);
    write_int_slice!(/// Output `&[u8]`.
        u8, write_u8_slice, write_u8);
    write_int_slice!(/// Output `&[u16]`.
        u16, write_u16_slice, write_u16);
    write_int_slice!(/// Output `&[u32]`.
        u32, write_u32_slice, write_u32);
    write_int_slice!(/// Output `&[u64]`.
        u64, write_u64_slice, write_u64);
    write_int_slice!(/// Output `&[usize]`.
        usize, write_usize_slice, write_usize);
    write_float_slice!(/// Output `&[f32]`.
        /// Each element is output to `nodp` decimal places.
        f32, write_f32_slice, write_f32);
    write_float_slice!(/// Output `&[f64]`.
        /// Each element is output to `nodp` decimal places.
        f64, write_f64_slice, write_f64);
    write_float_slice!(/// Output `&[f32]`.
        /// Each element is output to `nodp` decimal places in exponential format.
        f32, write_f32_slice_exp, write_f32_exp);
    write_float_slice!(/// Output `&[f64]`.
        /// Each element is output to `nodp` decimal places in exponential format.
        f64, write_f64_slice_exp, write_f64_exp);

    writeln_str_int!(/// Output `&str`.
        /// Then break the line.
        &str, writeln_str, write_str);
    writeln_str_int!(/// Output `i8`.
        /// Then break the line.
        i8, writeln_i8, write_i8);
    writeln_str_int!(/// Output `i16`.
        /// Then break the line.
        i16, writeln_i16, write_i16);
    writeln_str_int!(/// Output `i32`.
        /// Then break the line.
        i32, writeln_i32, write_i32);
    writeln_str_int!(/// Output `i64`.
        /// Then break the line.
        i64, writeln_i64, write_i64);
    writeln_str_int!(/// Output `isize`.
        /// Then break the line.
        isize, writeln_isize, write_isize);
    writeln_str_int!(/// Output `u8`.
        /// Then break the line.
        u8, writeln_u8, write_u8);
    writeln_str_int!(/// Output `u16`.
        /// Then break the line.
        u16, writeln_u16, write_u16);
    writeln_str_int!(/// Output `u32`.
        /// Then break the line.
        u32, writeln_u32, write_u32);
    writeln_str_int!(/// Output `u64`.
        /// Then break the line.
        u64, writeln_u64, write_u64);
    writeln_str_int!(/// Output `usize`.
        /// Then break the line.
        usize, writeln_usize, write_usize);
    writeln_float!(/// Output `f32` to `nodp` decimal places.
        /// Then break the line.
        f32, writeln_f32, write_f32);
    writeln_float!(/// Output `f64` to `nodp` decimal places.
        /// Then break the line.
        f64, writeln_f64, write_f64);
    writeln_float!(/// Output `f32` to `nodp` decimal places in exponential format.
        /// Then break the line.
        f32, writeln_f32_exp, write_f32_exp);
    writeln_float!(/// Output `f64` to `nodp` decimal places in exponential format.
        /// Then break the line.
        f64, writeln_f64_exp, write_f64_exp);
    writeln_str_int!(/// Output `&[i8]`.
        /// Then break the line.
        &[i8], writeln_i8_slice, write_i8_slice);
    writeln_str_int!(/// Output `&[i16]`.
        /// Then break the line.
        &[i16], writeln_i16_slice, write_i16_slice);
    writeln_str_int!(/// Output `&[i32]`.
        /// Then break the line.
        &[i32], writeln_i32_slice, write_i32_slice);
    writeln_str_int!(/// Output `&[i64]`.
        /// Then break the line.
        &[i64], writeln_i64_slice, write_i64_slice);
    writeln_str_int!(/// Output `&[isize]`.
        /// Then break the line.
        &[isize], writeln_isize_slice, write_isize_slice);
    writeln_str_int!(/// Output `&[u8]`.
        /// Then break the line.
        &[u8], writeln_u8_slice, write_u8_slice);
    writeln_str_int!(/// Output `&[u16]`.
        /// Then break the line.
        &[u16], writeln_u16_slice, write_u16_slice);
    writeln_str_int!(/// Output `&[u32]`.
        /// Then break the line.
        &[u32], writeln_u32_slice, write_u32_slice);
    writeln_str_int!(/// Output `&[u64]`.
        /// Then break the line.
        &[u64], writeln_u64_slice, write_u64_slice);
    writeln_str_int!(/// Output `&[usize]`.
        /// Then break the line.
        &[usize], writeln_usize_slice, write_usize_slice);
    writeln_float!(/// Output `&[f32]`.
        /// Each element is output to `nodp` decimal places.
        /// Then break the line.
        &[f32], writeln_f32_slice, write_f32_slice);
    writeln_float!(/// Output `&[f64]`.
        /// Each element is output to `nodp` decimal places.
        /// Then break the line.
        &[f64], writeln_f64_slice, write_f64_slice);
    writeln_float!(/// Output `&[f32]`.
        /// Each element is output to `nodp` decimal places in exponential format.
        /// Then break the line.
        &[f32], writeln_f32_slice_exp, write_f32_slice_exp);
    writeln_float!(/// Output `&[f64]`.
        /// Each element is output to `nodp` decimal places in exponential format.
        /// Then break the line.
        &[f64], writeln_f64_slice_exp, write_f64_slice_exp);
}