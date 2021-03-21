use core::convert::TryFrom;
use embedded_sdmmc::{TimeSource, Timestamp};
use k210_hal::rtc::{DateTime, Rtc};

pub struct RtcSource {
    rtc: Rtc,
}

impl RtcSource {
    pub fn new(rtc: Rtc) -> Self {
        Self { rtc }
    }
}

impl TimeSource for RtcSource {
    fn get_timestamp(&self) -> Timestamp {
        let DateTime {
            year,
            month,
            day,
            hours,
            minutes,
            seconds,
            ..
        } = self.rtc.timer_get().unwrap();
        Timestamp::from_calendar(
            u16::try_from(year).unwrap(),
            u8::try_from(month).unwrap(),
            u8::try_from(day).unwrap(),
            u8::try_from(hours).unwrap(),
            u8::try_from(minutes).unwrap(),
            u8::try_from(seconds).unwrap(),
        )
        .unwrap()
    }
}
