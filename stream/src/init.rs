use k210_hal::fpioa::IoPin;
use k210_hal::pac;

pub fn io(fpioa: k210_hal::fpioa::Parts) {
    use k210_hal::fpioa::functions::*;
    use k210_hal::fpioa::Pull;

    fpioa.io47.into_function(CMOS_PCLK);
    fpioa.io46.into_function(CMOS_XCLK);
    fpioa.io45.into_function(CMOS_HREF);
    fpioa.io44.into_function(CMOS_PWDN);
    fpioa.io43.into_function(CMOS_VSYNC);
    fpioa.io42.into_function(CMOS_RST);
    fpioa.io41.into_function(SCCB_SCLK);
    fpioa.io40.into_function(SCCB_SDA);

    fpioa.io39.into_function(SPI0_SCLK);
    fpioa.io36.into_function(SPI0_SS3);

    let mut hs1 = fpioa.io38.into_function(GPIOHS2); // dc
    let mut hs2 = fpioa.io37.into_function(GPIOHS3); // rst

    hs1.set_io_pull(Pull::Down);
    hs2.set_io_pull(Pull::Down);

    // Channel 0 = FPIO15
    // Channel 1 = FPIO14
    // Channel 2 = FPIO13
    // Channel 3 = FPIO12
    // Channel 4 = FPIO11

    fpioa.io15.into_function(SPI0_D0);
    fpioa.io14.into_function(SPI0_D1);
    fpioa.io13.into_function(SPI0_SCLK);
    fpioa.io12.into_function(SPI0_SS3);

    let mut hs3 = fpioa.io11.into_function(GPIOHS2); // dc
    hs3.set_io_pull(Pull::Down);

    // Include power mode selection in sysctl instead.
    unsafe {
        let ptr = pac::SYSCTL::ptr();
        (*ptr).misc.modify(|_, w| w.spi_dvp_data_enable().set_bit());
        (*ptr)
            .power_sel
            .modify(|_, w| w.power_mode_sel6().set_bit().power_mode_sel7().set_bit());
    }
}
