#![allow(dead_code)]
use core::cell::RefCell;

use core::result::Result;
use embedded_sdmmc::{Block, BlockCount, BlockDevice, BlockIdx};
use k210_hal::clock::Clocks;
use k210_hal::gpiohs::{Floating, Gpiohs0, Output};
use k210_hal::prelude::*;
use k210_hal::sleep::usleep;
use k210_hal::spi::{Spi, Spi01, *};
use k210_hal::time::Hertz;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    GoIdleState = 00,
    SendOpCond = 01,
    SendIfCond = 08,
    SendCsd = 09,
    SendCid = 10,
    StopTransmission = 12,
    SetBlockLength = 16,
    ReadSingleBlock = 17,
    ReadMultipleBlock = 18,
    SetBlockCount = 23,
    WriteBlock = 24,
    WriteMultipleBlock = 25,
    AppCmd = 55,
    ReadOcr = 58,
    AppSendOpCond = 41,
    // SetWrBlockEraseCount = 23,
}

pub struct SdCard<SPI> {
    spi: RefCell<Spi<SPI>>,
    cs: RefCell<Gpiohs0<Output<Floating>>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SdCardError {
    InitFailed,
    ReadCsdFailed,
    ReadDataFailed,
    CrcError,
    WriteError,
    Unknown,
}

impl<SPI: Spi01> SdCard<SPI> {
    pub fn new(spi: Spi<SPI>, cs: Gpiohs0<Output<Floating>>) -> SdCard<SPI> {
        SdCard {
            spi: RefCell::new(spi),
            cs: RefCell::new(cs),
        }
    }

    pub fn init(&mut self, clocks: &Clocks) {
        self.spi.borrow_mut().set_clk_rate(Hertz(200000), &clocks);
        self.spi.borrow_mut().configure(
            WorkMode::MODE0,
            FrameFormat::STANDARD,
            8,
            0,
            8,
            0,
            0,
            Aitm::AS_FRAME_FORMAT,
            Tmod::TRANS,
        );

        usleep(2000);

        self.cs.borrow_mut().try_set_high().unwrap();
        self.spi.borrow_mut().set_slave_select(Some(3));
        self.spi
            .borrow_mut()
            .try_send(&[0xff as u32; 10] as &[u32])
            .unwrap();

        self.send_command(Cmd::GoIdleState, 0x00, 0x95);
        if self.read_response() != 0x01 {
            panic!("[sd] Error: Unknown card (CMD0 failed).");
        }

        self.send_command(Cmd::SendIfCond, 0x01AA, 0x87);
        if self.read_response() != 0x01 {
            panic!("[sd] Error: Unknown card (CMD8 failed).");
        }
        if self.read_trailing_data() & 0xffffff != 0x1aa {
            panic!("[sd] Error: 0x1AA pattern check bit mismatched.");
        }

        let mut word = 0x01;
        while word == 0x01 {
            self.send_command(Cmd::AppCmd, 0x00, 0x00);
            self.read_response();
            self.send_command(Cmd::AppSendOpCond, 0x40000000, 0x00);
            word = self.read_response();
        }

        self.send_command(Cmd::ReadOcr, 0x00, 0x00);
        self.read_response();
        let data = self.read_trailing_data();
        let ccs = (data >> 30) & 0x1;

        if ccs == 0 {
            panic!("[sd] Error: Not using block address");
        }

        self.spi
            .borrow_mut()
            .set_clk_rate(Hertz(10_000_000), &clocks);
    }

    pub fn send_command(&self, cmd: Cmd, arg: u32, crc: u8) {
        self.spi.borrow_mut().configure(
            WorkMode::MODE0,
            FrameFormat::STANDARD,
            8,
            0,
            8,
            0,
            0,
            Aitm::AS_FRAME_FORMAT,
            Tmod::TRANS,
        );
        self.cs.borrow_mut().try_set_high().unwrap();
        self.spi.borrow_mut().try_send(0xff as u8).unwrap();
        self.cs.borrow_mut().try_set_low().unwrap();
        self.spi
            .borrow_mut()
            .try_send(&[
                cmd as u32 | 0x40,
                (arg >> 24) & 0xff,
                (arg >> 16) & 0xff,
                (arg >> 8) & 0xff,
                arg & 0xff,
                crc as u32 | 0x01,
            ] as &[u32])
            .unwrap();
    }

    pub fn send_data(&self, data: &[u8]) {
        self.spi.borrow_mut().configure(
            WorkMode::MODE0,
            FrameFormat::STANDARD,
            8,
            0,
            8,
            0,
            0,
            Aitm::AS_FRAME_FORMAT,
            Tmod::TRANS,
        );
        self.spi.borrow_mut().try_send(data).unwrap();
    }

    pub fn read_response(&self) -> u8 {
        self.spi.borrow_mut().configure(
            WorkMode::MODE0,
            FrameFormat::STANDARD,
            8,
            0,
            0,
            0,
            0,
            Aitm::AS_FRAME_FORMAT,
            Tmod::RECV,
        );

        let mut c: u8 = 0;
        let mut word: u8 = 0xff;
        while c < 8 && word == 0xff {
            word = self.spi.borrow_mut().try_read().unwrap();
            c += 1;
        }

        word
    }

    pub fn read_trailing_data(&self) -> u32 {
        self.spi.borrow_mut().configure(
            WorkMode::MODE0,
            FrameFormat::STANDARD,
            8,
            0,
            0,
            0,
            0,
            Aitm::AS_FRAME_FORMAT,
            Tmod::RECV,
        );

        let d0: u32 = self.spi.borrow_mut().try_read().unwrap();
        let d1: u32 = self.spi.borrow_mut().try_read().unwrap();
        let d2: u32 = self.spi.borrow_mut().try_read().unwrap();
        let d3: u32 = self.spi.borrow_mut().try_read().unwrap();
        let data: u32 = (d0 << 24) | (d1 << 16) | (d2 << 8) | d3;

        data
    }

    pub fn read_data<X: Trunc32>(&self, rx: &mut [X]) -> Result<(), SdCardError> {
        self.spi.borrow_mut().configure(
            WorkMode::MODE0,
            FrameFormat::STANDARD,
            8,
            0,
            8,
            0,
            0,
            Aitm::AS_FRAME_FORMAT,
            Tmod::RECV,
        );

        let mut token: u8 = 0xff;
        while (token & 0b11) == 0b11 {
            token = self.read_response();
        }

        if let Ok(_) = self.spi.borrow_mut().recv_data(rx) {
            Ok(())
        } else {
            Err(SdCardError::ReadDataFailed)
        }
    }

    pub fn wait_ready(&self) {
        self.spi.borrow_mut().configure(
            WorkMode::MODE0,
            FrameFormat::STANDARD,
            8,
            0,
            0,
            0,
            0,
            Aitm::AS_FRAME_FORMAT,
            Tmod::RECV,
        );

        let mut c = 0x00 as u8;
        while c == 0x00 {
            c = self.spi.borrow_mut().try_read().unwrap();
        }
    }
}

impl<SPI: Spi01> BlockDevice for SdCard<SPI> {
    type Error = SdCardError;

    fn read(
        &self,
        blocks: &mut [Block],
        start_block_idx: BlockIdx,
        _reason: &str,
    ) -> Result<(), Self::Error> {
        if blocks.len() == 1 {
            self.send_command(Cmd::ReadSingleBlock, start_block_idx.0, 0x00);
            self.read_response();
            self.read_data(&mut blocks[0].contents)?;
            self.read_response();
            self.read_response();
        } else {
            self.send_command(Cmd::ReadMultipleBlock, start_block_idx.0, 0x00);
            self.read_response();
            for i in 0..blocks.len() {
                self.read_data(&mut blocks[i].contents)?;
                self.read_response();
                self.read_response();
            }
            self.send_command(Cmd::StopTransmission, 0x00, 0x00);
            self.read_response();
            self.read_response();
        }

        Ok(())
    }

    fn write(&self, blocks: &[Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        if blocks.len() == 1 {
            self.send_command(Cmd::WriteBlock, start_block_idx.0, 0x00);
            self.read_response();
            self.send_data(&[0xff, 0xfe]);
            self.send_data(&blocks[0].contents);
            self.send_data(&[0x00, 0x00]);
            let status = self.read_response() & 0x1f;
            if status == 0b1011 {
                return Err(SdCardError::CrcError);
            }
            if status == 0b1101 {
                return Err(SdCardError::WriteError);
            }
            if status != 0b101 {
                return Err(SdCardError::Unknown);
            }
            self.wait_ready();
        } else {
            self.send_command(Cmd::WriteMultipleBlock, start_block_idx.0, 0x00);
            self.read_response();
            for i in 0..blocks.len() {
                self.send_data(&[0xff, 0xfc]);
                self.send_data(&blocks[i].contents);
                self.send_data(&[0x00, 0x00]);
                let status = self.read_response() & 0x1f;
                if status == 0b1011 {
                    return Err(SdCardError::CrcError);
                }
                if status == 0b1101 {
                    return Err(SdCardError::WriteError);
                }
                if status != 0b101 {
                    return Err(SdCardError::Unknown);
                }
                self.wait_ready();
            }
            self.send_data(&[0xfd, 0x00]);
            self.wait_ready();
        }

        Ok(())
    }

    fn num_blocks(&self) -> Result<BlockCount, Self::Error> {
        self.send_command(Cmd::SendCsd, 0x00, 0x00);
        if self.read_response() != 0x00 {
            return Err(SdCardError::ReadCsdFailed);
        }

        let mut rx: [u8; 18] = [0; 18];
        self.read_data(&mut rx).unwrap();

        // Only support SDC v2 for now
        let c_size: u32 = u32::from(rx[7] & 0x3f) << 16 | u32::from(rx[8]) << 8 | u32::from(rx[9]);

        Ok(BlockCount((c_size + 1) * 1000))
    }
}
