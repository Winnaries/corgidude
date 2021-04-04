#![no_std]
#![no_main]

use core::str::FromStr;

use embedded_sdmmc::{Controller, Mode, VolumeIdx};
use k210_hal::dmac::{DmacChannel, DmacExt};
use k210_hal::dvp::DvpExt;
use k210_hal::prelude::*;
use k210_hal::rtc::RtcExt;
use k210_hal::stdout::Stdout;
use k210_hal::time::Hertz;
use k210_hal::{dvp, pac, spi};
use riscv_rt::entry;
use spi::SpiExt;

mod init;
mod lcd;
mod ov2640;
mod panic;
mod rtc_source;
mod sdcard;

// TODO
// 1. Fix up the SPI interface
// 2. Implement SD card interface
// 3. Get to write the text file through the DMA

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

    /* Enable PLL0 and freeze the clocks */
    let mut sysctl = p.SYSCTL.constrain();
    sysctl.pll0.set_frequency(Hertz(800_000_000));
    let clock = sysctl.clocks();

    /* Setup default stdout for debugging */
    let serial = p.UARTHS.configure((115_200 as u32).bps(), &clock);
    let (mut tx, _) = serial.split();
    let mut stdout = Stdout(&mut tx);

    /* Correct the FPIOA routing */
    let fpioa = p.FPIOA.split(&mut sysctl.apb0);
    init::io(fpioa);

    /* Configure DVP periperals */
    let dvp = p.DVP.constrain();
    dvp.init();
    let (mid, pid) = ov2640::read_id(&dvp);
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

    /* Configure LCD using with DMAC */
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

    /* Configuring SD Card interface */
    let spi1 = p.SPI1.constrain(&mut sysctl.apb2);
    let gpio0 = p.GPIOHS.split().gpiohs0.into_output();
    let mut sdcard = sdcard::SdCard::new(spi1, gpio0);
    sdcard.init(&clock);

    /* Configuring real-time clock */
    let mut rtc = p.RTC.constrain(&mut sysctl.apb1);
    rtc.init(&clock);
    rtc.timer_set(2020, 3, 21, 20, 47, 00, &clock).unwrap();
    let rtc = rtc_source::RtcSource::new(rtc);

    let mut sd = Controller::new(sdcard, rtc);

    let mut volume = sd.get_volume(VolumeIdx(0)).unwrap();
    let root = sd.open_root_dir(&volume).unwrap();
    let central = sd.open_dir(&volume, &root, "1");

    writeln!(stdout, "{:?}", &central).unwrap();

    // let mut file = sd
    //     .open_file_in_dir(&mut volume, &root, "test.txt", Mode::ReadWriteCreate)
    //     .unwrap();

    // let mut buffer: [u8; 100] = [0; 100];
    // // sd.write(&mut volume, &mut file, buffer).unwrap();
    // sd.read(&volume, &mut file, &mut buffer).unwrap();

    // writeln!(stdout, "{:?}", buffer).unwrap();

    loop {
        dvp.get_image();
        lcd.set_image(unsafe { &FRAME.image });
    }
}
