use k210_hal::clock::Clocks;
use k210_hal::dmac::{Dmac, DmacChannel};
use k210_hal::gpiohs::GpiohsAccess;
use k210_hal::pac::GPIOHS;
use k210_hal::prelude::*;
use k210_hal::sleep::usleep;
use k210_hal::spi::{Spi, Spi01};

const CLOCK_RATE: u32 = 18_000_000;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Command {
    NOP = 0x00,
    SWRESET = 0x01,
    RDDID = 0x04,
    RDDST = 0x09,
    RDDPM = 0x0A,
    RDDMADCTL = 0x0B,
    RDDCOLMOD = 0x0C,
    RDDIM = 0x0D,
    RDDSM = 0x0E,
    RDDSDR = 0x0F,
    SLPIN = 0x10,
    SLPOUT = 0x11,
    PTLON = 0x12,
    NORON = 0x13,
    INVOF = 0x20,
    INVON = 0x21,
    GAMSET = 0x26,
    DISPOFF = 0x28,
    DISPON = 0x29,
    CASET = 0x2A,
    RASET = 0x2B,
    RAMWR = 0x2C,
    RAMRD = 0x2E,
    PTLAR = 0x30,
    VSCRDEF = 0x33,
    TEOFF = 0x34,
    TEON = 0x35,
    MADCTL = 0x36,
    VSCRSADD = 0x37,
    IDMOFF = 0x38,
    IDMON = 0x39,
    COLMOD = 0x3A,
    RAMWRC = 0x3C,
    RAMRDC = 0x3E,
    TESCAN = 0x44,
    RDTESCAN = 0x45,
    WRDISBV = 0x51,
    RDDISBV = 0x52,
    WRCTRLD = 0x53,
    RDCTRLD = 0x54,
    WRCACE = 0x55,
    RDCABC = 0x56,
    WRCABCMB = 0x5E,
    RDCABCMB = 0x5F,
    RDABCSDR = 0x68,
    RAMCTRL = 0xB0,
    RGBCTRL = 0xB1,
    PORCTRL = 0xB2,
    FRCTRL1 = 0xB3,
    PARCTRL = 0xB5,
    GCTRL = 0xB7,
    GTADJ = 0xB8,
    DGMEN = 0xBA,
    VCOMS = 0xBB,
    LCMCTRL = 0xC0,
    IDSET = 0xC1,
    VDVVRHEN = 0xC2,
    VRHS = 0xC3,
    VDVSET = 0xC4,
    VCMOFSET = 0xC5,
    FRCTR2 = 0xC6,
    CABCCTRL = 0xC7,
    REGSEL1 = 0xC8,
    REGSEL2 = 0xCA,
    PWMFRSEL = 0xCC,
    PWCTRL1 = 0xD0,
    VAPVANEN = 0xD2,
    RDID1 = 0xDA,
    RDID2 = 0xDB,
    RDID3 = 0xDC,
    CMD2EN = 0xDF,
    PVGAMCTRL = 0xE0,
    NVGAMCTRL = 0xE1,
    DGMLUTR = 0xE2,
    DGMLUTB = 0xE3,
    GATECTRL = 0xE4,
    SPI2EN = 0xE7,
    PWCTRL2 = 0xE8,
    EQCTRL = 0xE9,
    PROMCTRL = 0xEC,
    PROMEN = 0xFA,
    NVMSET = 0xFC,
    PROMACT = 0xFE,
}

#[repr(u8)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Direction {
    XY_RLUD = 0x00,
    YX_RLUD = 0x20,
    XY_LRUD = 0x40,
    YX_LRUD = 0x60,
    XY_RLDU = 0x80,
    YX_RLDU = 0xA0,
    XY_LRDU = 0xC0,
    YX_LRDU = 0xE0,
}

pub struct Lcd<SPI> {
    dmac: Dmac,
    channel: DmacChannel,
    spi: Spi<SPI>,
    cs_num: u8,
    dc_gpio: u32,
    rs_gpio: u32,
}

impl<SPI: Spi01> Lcd<SPI> {
    pub fn new(
        dmac: Dmac,
        channel: DmacChannel,
        spi: Spi<SPI>,
        cs_num: u8,
        dc_gpio: u32,
        rs_gpio: u32,
    ) -> Self {
        Self {
            dmac,
            channel,
            spi,
            cs_num,
            dc_gpio,
            rs_gpio,
        }
    }

    fn init_rst(&mut self) {
        GPIOHS::set_output_en(self.rs_gpio as usize, true);
        GPIOHS::set_input_en(self.rs_gpio as usize, false);
        GPIOHS::set_output_value(self.rs_gpio as usize, true);
    }

    fn init_dcx(&mut self) {
        GPIOHS::set_output_en(self.dc_gpio as usize, true);
        GPIOHS::set_input_en(self.dc_gpio as usize, false);
        GPIOHS::set_output_value(self.dc_gpio as usize, true);
    }

    fn set_dcx_control(&mut self) {
        GPIOHS::set_output_value(self.dc_gpio as usize, false);
    }

    fn set_dcx_data(&mut self) {
        GPIOHS::set_output_value(self.dc_gpio as usize, true);
    }

    fn set_reset(&mut self, val: bool) {
        GPIOHS::set_output_value(self.rs_gpio as usize, val);
    }

    fn write_command(&mut self, cmd: Command) {
        use k210_hal::spi::{Aitm, FrameFormat, Tmod, WorkMode};
        self.set_dcx_control();
        self.spi.configure(
            WorkMode::MODE0,
            FrameFormat::OCTAL,
            8,
            0,
            8,
            0,
            0,
            Aitm::AS_FRAME_FORMAT,
            Tmod::TRANS,
        );
        self.spi.set_slave_select(Some(self.cs_num));
        self.spi.try_send(cmd as u32).unwrap();
    }

    fn write_byte(&mut self, buf: &[u32]) {
        use k210_hal::spi::{Aitm, FrameFormat, Tmod, WorkMode};
        self.set_dcx_data();
        self.spi.configure(
            WorkMode::MODE0,
            FrameFormat::OCTAL,
            8,
            0,
            8,
            0,
            0,
            Aitm::AS_FRAME_FORMAT,
            Tmod::TRANS,
        );
        self.spi.set_slave_select(Some(self.cs_num));
        self.spi
            .send_data_dma(&mut self.dmac, self.channel, buf)
            .unwrap();
    }

    pub fn set_area(&mut self, height: u32, width: u32) {
        self.write_command(Command::CASET);
        self.write_byte(&[
            ((320 - width) as u32 >> 8).into(),
            ((320 - width) as u32 & 0xff).into(),
            ((320 - 1) as u32 >> 8).into(),
            ((320 - 1) as u32 & 0xff).into(),
        ]);
        self.write_command(Command::RASET);
        self.write_byte(&[
            0x00,
            0x00,
            ((height - 1) >> 8).into(),
            ((height - 1) & 0xff).into(),
        ]);
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.write_command(Command::MADCTL);
        self.write_byte(&[direction as u32]);
    }

    pub fn init(&mut self, clocks: &Clocks) {
        use k210_hal::spi::{Aitm, FrameFormat, Tmod, WorkMode};
        self.init_dcx();
        self.init_rst();

        self.set_reset(false);
        self.spi.set_clk_rate(CLOCK_RATE.hz(), clocks);
        self.spi.configure(
            WorkMode::MODE0,
            FrameFormat::OCTAL,
            8,
            0,
            8,
            0,
            0,
            Aitm::AS_FRAME_FORMAT,
            Tmod::TRANS,
        );
        self.set_reset(true);

        self.write_command(Command::SWRESET);
        usleep(1000000);
        self.write_command(Command::SLPOUT);
        usleep(1000000);

        self.write_command(Command::RAMCTRL);
        self.write_byte(&[0x00, 0xf0 | 0x08]);

        self.write_command(Command::COLMOD);
        self.write_byte(&[0x55]);

        self.write_command(Command::DISPON);

        self.set_area(240, 320);

        self.write_command(Command::INVON);

        self.set_direction(Direction::YX_LRUD);
    }

    pub fn set_image(&mut self, data: &[u32]) {
        use k210_hal::spi::{Aitm, FrameFormat, Tmod, WorkMode};
        self.write_command(Command::RAMWR);
        self.set_dcx_data();
        self.spi.configure(
            WorkMode::MODE0,
            FrameFormat::OCTAL,
            32, /*data bits*/
            1,  /*endian*/
            0,  /*instruction length*/
            32, /*address length*/
            0,  /*wait cycles*/
            Aitm::AS_FRAME_FORMAT,
            Tmod::TRANS,
        );
        self.spi.set_slave_select(Some(self.cs_num));
        self.spi
            .send_data_dma(&mut self.dmac, self.channel, data)
            .unwrap();
    }
}
