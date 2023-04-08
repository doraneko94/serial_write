//! 

#![no_std]
#![no_main]

use panic_halt as _;
use rp2040_hal as hal;
use hal::pac;
use rp2040_hal::clocks::Clock;

use usb_device::class_prelude::{UsbBus, UsbError};
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

use serial_write::Writer;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

macro_rules! write_check {
    ($truth: expr, $val: expr, $f: ident, $writer: ident, $serial: ident) => {
        let _ = $writer.write_str($truth, &mut $serial);
        match $writer.$f($val, &mut $serial) {
            Ok(n) => {
                let _ = $writer.write_str(" (", &mut $serial);
                let _ = $writer.write_usize(n, &mut $serial);
                let _ = $writer.writeln_str(" bits).", &mut $serial);
            }
            Err((_, n)) => {
                let _ = $writer.write_str(" (Error; ", &mut $serial);
                let _ = $writer.write_usize(n, &mut $serial);
                let _ = $writer.writeln_str(" bits).", &mut $serial);
            }
        }
    };
}
macro_rules! write_check_float {
    ($truth: expr, $val: expr, $nodp: expr, $f: ident, $writer: ident, $serial: ident) => {
        let _ = $writer.write_str($truth, &mut $serial);
        match $writer.$f($val, $nodp, &mut $serial) {
            Ok(n) => {
                let _ = $writer.write_str(" (", &mut $serial);
                let _ = $writer.write_usize(n, &mut $serial);
                let _ = $writer.writeln_str(" bits).", &mut $serial);
            }
            Err((_, n)) => {
                let _ = $writer.write_str(" (Error; ", &mut $serial);
                let _ = $writer.write_usize(n, &mut $serial);
                let _ = $writer.writeln_str(" bits).", &mut $serial);
            }
        }
    };
}

#[rp2040_hal::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Set the USB bus
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set the serial port
    let mut serial = SerialPort::new(&usb_bus);

    // Set a USB device
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(2)
        .build();

    let mut writer = Writer::new();

    loop {
        for _ in 0..200 {
            delay.delay_ms(5);
            let _ = usb_dev.poll(&mut [&mut serial]);
        }

        let _ = writer.writeln_str("===============================================", &mut serial);
        write_check!("127 [u8]: ", i8::MAX, write_i8, writer, serial);
        write_check!("-128 [u8]: ", i8::MIN, write_i8, writer, serial);
        write_check!("32767 [u16]: ", i16::MAX, write_i16, writer, serial);
        write_check!("-32768 [u16]: ", i16::MIN, write_i16, writer, serial);
        write_check!("2147483647 [u32]: ", i32::MAX, write_i32, writer, serial);
        write_check!("-2147483648 [u32]: ", i32::MIN, write_i32, writer, serial);
        write_check!("9223372036854775807 [u64]: ", i64::MAX, write_i64, writer, serial);
        write_check!("-9223372036854775808 [u64]: ", i64::MIN, write_i64, writer, serial);
        write_check!("2147483647 [usize]: ", isize::MAX, write_isize, writer, serial);
        write_check!("-2147483648 [usize]: ", isize::MIN, write_isize, writer, serial);
        let _ = writer.writeln_str("===============================================", &mut serial);
        
        for _ in 0..200 {
            delay.delay_ms(5);
            let _ = usb_dev.poll(&mut [&mut serial]);
        }

        let _ = writer.writeln_str("===============================================", &mut serial);
        write_check!("255 [u8]: ", u8::MAX, write_u8, writer, serial);
        write_check!("0 [u8]: ", u8::MIN, write_u8, writer, serial);
        write_check!("65535 [u16]: ", u16::MAX, write_u16, writer, serial);
        write_check!("0 [u16]: ", u16::MIN, write_u16, writer, serial);
        write_check!("4294967295 [u32]: ", u32::MAX, write_u32, writer, serial);
        write_check!("0 [u32]: ", u32::MIN, write_u32, writer, serial);
        write_check!("18446744073709551615 [u64]: ", u64::MAX, write_u64, writer, serial);
        write_check!("0 [u64]: ", u64::MIN, write_u64, writer, serial);
        write_check!("4294967295 [usize]: ", usize::MAX, write_usize, writer, serial);
        write_check!("0 [usize]: ", usize::MIN, write_usize, writer, serial);
        let _ = writer.writeln_str("===============================================", &mut serial);

        for _ in 0..200 {
            delay.delay_ms(5);
            let _ = usb_dev.poll(&mut [&mut serial]);
        }
        
        let _ = writer.writeln_str("===============================================", &mut serial);
        write_check_float!(
            "340282350000000000000000000000000000000.0 [f32]: ",
            f32::MAX, 1, write_f32, writer, serial
        );
        write_check_float!(
            "-340282350000000000000000000000000000000.0 [f32]: ",
            f32::MIN, 1, write_f32, writer, serial
        );
        write_check_float!(
            "0.00000011920929 [f32]: ",
            f32::EPSILON, 7, write_f32, writer, serial
        );
        write_check_float!(
            "179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0 [f32]: ",
            f64::MAX, 1, write_f64, writer, serial
        );
        write_check_float!(
            "-179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0 [f32]: ",
            f64::MIN, 1, write_f64, writer, serial
        );

    }
}

// End of file