use k210_hal::fpioa::IoPin;
use k210_hal::pac;

// Logic Analyzer probes
// Channel 0 = FPIO15
// Channel 1 = FPIO14
// Channel 2 = FPIO13
// Channel 3 = FPIO12
// Channel 4 = FPIO11

pub fn io(fpioa: k210_hal::fpioa::Parts) {
    use k210_hal::fpioa::functions::*;
    use k210_hal::fpioa::Pull;

    // Camera
    fpioa.io47.into_function(CMOS_PCLK);
    fpioa.io46.into_function(CMOS_XCLK);
    fpioa.io45.into_function(CMOS_HREF);
    fpioa.io44.into_function(CMOS_PWDN);
    fpioa.io43.into_function(CMOS_VSYNC);
    fpioa.io42.into_function(CMOS_RST);
    fpioa.io41.into_function(SCCB_SCLK);
    fpioa.io40.into_function(SCCB_SDA);

    // LCD
    fpioa.io39.into_function(SPI0_SCLK);
    fpioa.io36.into_function(SPI0_SS3);
    fpioa.io38.into_function(GPIOHS2).set_io_pull(Pull::Down); // dc
    fpioa.io37.into_function(GPIOHS3).set_io_pull(Pull::Down); // rs

    // SDMMC
    fpioa.io26.into_function(SPI1_D1);
    fpioa.io27.into_function(SPI1_SCLK);
    fpioa.io28.into_function(SPI1_D0);
    fpioa.io29.into_function(GPIOHS0).set_io_pull(Pull::Down); // sdc ss

    // // Logic Analyzer
    // fpioa.io15.into_function(SPI1_D1);
    // fpioa.io14.into_function(SPI1_SCLK);
    // fpioa.io13.into_function(SPI1_D0);
    // fpioa.io12.into_function(GPIOHS0).set_io_pull(Pull::Down);

    // Include power mode selection in sysctl instead.
    unsafe {
        let ptr = pac::SYSCTL::ptr();
        (*ptr).misc.modify(|_, w| w.spi_dvp_data_enable().set_bit());
        (*ptr)
            .power_sel
            .modify(|_, w| w.power_mode_sel6().set_bit().power_mode_sel7().set_bit());
    }
}
