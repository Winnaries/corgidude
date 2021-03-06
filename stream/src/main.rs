#![no_std]
#![no_main]

use k210_hal::dmac::{DmacChannel, DmacExt};
use k210_hal::dvp::DvpExt;
use k210_hal::prelude::*;
use k210_hal::stdout::Stdout;
use k210_hal::{dvp, pac, spi};
use riscv_rt::entry;
use spi::SpiExt;

mod init;
mod lcd;
mod ov2640;
mod panic;

const DISP_PIXELS: usize = 320 * 240;
const COLOR: u32 = 0x00;

#[repr(C, align(64))]
struct ScreenRAM {
    pub image: [u32; DISP_PIXELS / 2],
}

impl ScreenRAM {
    pub fn as_mut_ptr(&mut self) -> *mut u32 {
        self.image.as_mut_ptr()
    }
}

static mut FRAME: ScreenRAM = ScreenRAM {
    image: [COLOR; DISP_PIXELS / 2],
};

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    // Freeze clocks frequency
    let mut sysctl = p.SYSCTL.constrain();
    let clock = sysctl.clocks();

    // Configure UARTHS for debugging purpose
    let serial = p.UARTHS.configure((115_200 as u32).bps(), &clock);
    let (mut tx, _) = serial.split();
    let mut stdout = Stdout(&mut tx);

    // Setup FPIOA
    let fpioa = p.FPIOA.split(&mut sysctl.apb0);
    init::io(fpioa);

    // Init DVP
    let dvp = p.DVP.constrain();
    dvp.init();

    // Testing SCCB interface and verifying device ID
    let (mid, pid) = ov2640::read_id(&dvp);
    writeln!(stdout, "[dvp] mid: {:02x}, pid: {:02x}", &mid, &pid).unwrap();

    if mid != 0x7fa2 || pid != 0x2642 {
        writeln!(stdout, "[dvp] manufacturer and product id mismatched").unwrap();
        panic!()
    }

    writeln!(stdout, "[dvp] setting xclk rate").unwrap();
    dvp.set_xclk_rate((24_000_000 as u32).hz(), &clock);

    writeln!(stdout, "[dvp] setting image format").unwrap();
    dvp.set_image_format(dvp::ImageFormat::RGB);

    writeln!(stdout, "[dvp] setting image size").unwrap();
    dvp.set_image_size(true, 320, 240);

    writeln!(stdout, "[dvp] disabling auto").unwrap();
    dvp.set_auto(false);

    writeln!(stdout, "[dvp] init OV2640 config").unwrap();
    ov2640::init(&dvp);

    writeln!(stdout, "[dvp] setting display address").unwrap();
    dvp.set_display_addr(unsafe { Some(FRAME.as_mut_ptr()) });

    writeln!(stdout, "[lcd] locking DMAC").unwrap();
    let mut dmac = p.DMAC.constrain();

    writeln!(stdout, "[lcd] initing DMAC").unwrap();
    dmac.init();

    writeln!(stdout, "[lcd] locking SPI0").unwrap();
    let spi0 = p.SPI0.constrain(&mut sysctl.apb2);

    writeln!(stdout, "[lcd] creating lcd instance").unwrap();
    let mut lcd = lcd::Lcd::new(dmac, DmacChannel::Channel0, spi0, 3, 2, 3);

    writeln!(stdout, "[lcd] flushing initial config").unwrap();
    lcd.init(&clock);

    writeln!(stdout, "[lcd] clearing the screen to {:04x}", &COLOR).unwrap();
    lcd.set_image(unsafe { &FRAME.image });

    loop {
        dvp.get_image();
        lcd.set_image(unsafe { &FRAME.image });
    }
}
