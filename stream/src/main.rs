#![no_std]
#![no_main]

use dvp::DvpExt;
use k210_hal::prelude::*;
use k210_hal::stdout::Stdout;
use k210_hal::{dvp, pac};
use riscv_rt::entry;

mod init;
mod ov2640;
mod panic;

const DISP_PIXELS: usize = 240 * 240;

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
    image: [0; DISP_PIXELS / 2],
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
    dvp.set_image_size(true, 240, 240);

    writeln!(stdout, "[dvp] init OV2640 config").unwrap();
    ov2640::init(&dvp);

    writeln!(stdout, "[dvp] setting display address").unwrap();
    dvp.set_display_addr(unsafe { Some(FRAME.as_mut_ptr()) });

    loop {
        dvp.get_image();
        writeln!(stdout, "[dvp] sample output: {:?}", unsafe {
            FRAME.image[500]
        })
        .unwrap();
    }
}
