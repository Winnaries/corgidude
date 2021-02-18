#![no_std]
#![no_main]

use board::dvp::DVPExt;
use board::fpioa::{self, function};
use board::gpiohs;
use board::sysctl;
use board::{def::io, gpio::direction};
use fpioa::pull;
use k210_hal::{pac, prelude::*, stdout::Stdout};
use riscv_rt::entry;

// Kendryte K210 = 400MHz

fn dvp_init() {
    fpioa::set_function(io::DVP_SDA, function::SCCB_SDA);
    fpioa::set_function(io::DVP_SCL, function::SCCB_SCLK);
    fpioa::set_function(io::DVP_RST, function::CMOS_RST);
    fpioa::set_function(io::DVP_VSYNC, function::CMOS_VSYNC);
    fpioa::set_function(io::DVP_HSYNC, function::CMOS_HREF);
    fpioa::set_function(io::DVP_PWDN, function::CMOS_PWDN);
    fpioa::set_function(io::DVP_XCLK, function::CMOS_XCLK);
    fpioa::set_function(io::DVP_PCLK, function::CMOS_PCLK);

    sysctl::set_spi0_dvp_data(true);
}

fn lcd_init() {
    fpioa::set_function(io::LCD_CS, function::SPI0_SS3);
    fpioa::set_function(io::LCD_WR, function::SPI0_SCLK);
    fpioa::set_function(io::LCD_DC, function::GPIOHS1);
    fpioa::set_io_pull(io::LCD_DC, pull::DOWN);
    fpioa::set_function(io::LCD_RST, function::GPIOHS2);
    fpioa::set_io_pull(io::LCD_RST, pull::DOWN);

    gpiohs::set_direction(1, direction::OUTPUT);
    gpiohs::set_direction(2, direction::OUTPUT);
}

/**
 * @TODO:
 * 1. Configure DVP
 *  - Set the SCCB
 *  - Set XCLK rate
 *  - Set image format
 *  - Set image size
 */

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let clock = k210_hal::clock::Clocks::new();

    let serial = p.UARTHS.configure((115_200 as u32).bps(), &clock);
    let (mut tx, _) = serial.split();

    let mut stdout = Stdout(&mut tx);

    writeln!(stdout, "[dvp] initiating").unwrap();
    dvp_init();
    writeln!(stdout, "[lcd] initiating").unwrap();
    lcd_init();

    let mut _dvp = p.DVP.constrain();

    loop {}
}
