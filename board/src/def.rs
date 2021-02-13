//! General board definitions for CorgiDude.

/** Dislay width in pixels */
pub const DISP_WIDTH: u16 = 240;

/** Dislay height in pixels */
pub const DISP_HEIGHT: u16 = 240;

/** Number of pixels on the screen */
pub const DISP_PIXELS: usize = (DISP_WIDTH as usize) * (DISP_HEIGHT as usize);

#[derive(Copy, Clone)]
pub enum io {
    /** JTAG TCK */
    JTAG_TCK = 0,
    /** JTAG TDI */
    JTAG_TDI = 1,
    /** JTAG TMS */
    JTAG_TMS = 2,
    /** JTAG TDO */
    JTAG_TDO = 3,
    /** Host RX (from STM32F103C8) */
    ISP_RX = 4,
    /** Host TX (to STM32F103C8) */
    ISP_TX = 5,
    /** WIFI serial TX (from perspective of ESP8285, so our RX) */
    WIFI_TX = 6,
    /** WIFI serial RX (from perspective of ESP8285, so our TX) */
    WIFI_RX = 7,
    /** WIFI enable (to ESP8285) */
    WIFI_EN = 8,
    /** Unused */
    IO9 = 9,
    /** Unused */
    IO10 = 10,
    /** General purpose I/O pin */
    IO11 = 11,
    /** Blue led (output) */
    IO12 = 12,
    /** Green led (output) */
    IO13 = 13,
    /** Red led (output) */
    IO14 = 14,
    /** Key direction 1 press (input) */
    IO15 = 15,
    /** Key center press (input) */
    BOOT = 16,
    /** Key direction 2 press (input) */
    IO17 = 17,
    /** Microphone I2S BCK */
    IO18 = 18,
    /** Microphone I2S WS */
    IO19 = 19,
    /** Microphone I2S DAT3 */
    IO20 = 20,
    /** Microphone I2S DAT2 */
    IO21 = 21,
    /** Red led */
    RGB_LED_R = 22,
    /** Blue led */
    RGB_LED_B = 23,
    /** Green led */
    RGB_LED_G = 24,
    /** Microphone LED CLK */
    IO25 = 25,
    /** SDCARD SPI MISO */
    IO26 = 26,
    /** SDCARD SPI SCLK */
    IO27 = 27,
    /** SDCARD SPI MOSI */
    IO28 = 28,
    /** SDCARD SPI CS */
    IO29 = 29,
    /** I2C bus 1 SCLK (NS2009, MSA300) */
    IO30 = 30,
    /** I2C bus 2 SDA (NS2009, MSA300) */
    IO31 = 31,
    /** General purpose I/O pin */
    IO32 = 32,
    /** DAC I2S WS */
    IO33 = 33,
    /** DAC I2S DA */
    IO34 = 34,
    /** DAC I2S BCK */
    IO35 = 35,
    /** LCD chip select (output) */
    LCD_CS = 36,
    /** LCD reset (output) */
    LCD_RST = 37,
    /** LCD Data/Command */
    LCD_DC = 38,
    /** LCD SPI SCLK */
    LCD_WR = 39,
    /** Camera DVP SDA */
    DVP_SDA = 40,
    /** Camera DVP SCL */
    DVP_SCL = 41,
    /** Camera DVP RST */
    DVP_RST = 42,
    /** Camera DVP VSYNC */
    DVP_VSYNC = 43,
    /** Camera DVP PWDN */
    DVP_PWDN = 44,
    /** Camera DVP HSYNC */
    DVP_HSYNC = 45,
    /** Camera DVP XCLK */
    DVP_XCLK = 46,
    /** Camera DVP PCLK */
    DVP_PCLK = 47,
}

impl From<io> for usize {
    fn from(io: io) -> Self {
        io as usize
    }
}
