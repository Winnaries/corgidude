use crate::sysctl;
use k210_hal::pac;

pub enum addr_len {
    W8,
    W16,
}

pub trait DVPExt: Sized {
    fn constrain(self) -> DVP;
}

impl DVPExt for pac::DVP {
    fn constrain(self) -> DVP {
        DVP {
            dvp: self,
            addr_len: addr_len::W8,
        }
    }
}

pub struct DVP {
    dvp: pac::DVP,
    addr_len: addr_len,
}

impl DVP {
    pub fn init(&self) {
        sysctl::clock_enable(sysctl::clock::DVP);
        sysctl::reset(sysctl::reset::DVP);
    }

    pub fn reset(&self) {
        self.dvp.cmos_cfg.write(|w| w.power_down().set_bit());
        self.dvp.cmos_cfg.write(|w| w.power_down().clear_bit());
        self.dvp.cmos_cfg.write(|w| w.reset().set_bit());
        self.dvp.cmos_cfg.write(|w| w.reset().clear_bit());
    }

    pub fn sccb_clock_init(&self) {
        unsafe {
            self.dvp
                .sccb_cfg
                .write(|w| w.scl_lcnt().bits(255).scl_hcnt().bits(255))
        }
    }

    pub fn sccb_set_clk_rate(&self) {
        let in_freq = sysctl::clock_get_freq(sysctl::clock::APB1);

        unsafe { self.dvp.s }
    }
}
