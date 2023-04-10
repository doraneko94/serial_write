//! Output tests for each numeric type.

#![no_std]
#![no_main]

use panic_halt as _;
use rp2040_hal as hal;
use hal::pac;
use rp2040_hal::clocks::Clock;

use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

use serial_write::Writer;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

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

    // Create `Writer` struct.
    let mut writer = Writer::new();

    loop {
        for _ in 0..200 {
            delay.delay_ms(5);
            let _ = usb_dev.poll(&mut [&mut serial]);
        }

        // Output "Hello, world!", and returns output size.
        let res = writer.writeln_str("Hello, world!", &mut serial);
        match res {
            Ok(n) => {
                let _ = writer.write_str("Success: Output ", &mut serial);
                let _ = writer.write_usize(n, &mut serial);
                let _ = writer.writeln_str(" bits.", &mut serial);
            }
            Err((_, n)) => {
                let _ = writer.write_str("Failed: Output ", &mut serial);
                let _ = writer.write_usize(n, &mut serial);
                let _ = writer.writeln_str(" bits.", &mut serial);
            }
        }

        // "12.34"
        let _ = writer.writeln_f32(12.345, 2, &mut serial);
        // "12.3450"
        let _ = writer.writeln_f32(12.345, 4, &mut serial);
        // " 1.23e001"
        let _ = writer.writeln_f32_exp(12.345, 2, &mut serial);
        // "[  1.2e000, -2.3e000,  3.4e000 ]"
        let _ = writer.writeln_f32_slice_exp(&[1.23, -2.34, 3.45], 1, &mut serial);
    }
}

// End of file