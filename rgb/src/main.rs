#![no_std]
#![no_main]

use board::def::io;
use board::fpioa::{self, function};
use board::gpio::direction;
use k210_hal::pac::GPIO;
use riscv_rt::entry;

fn fpioa_init() {
    fpioa::set_function(io::RGB_LED_R, function::GPIO0);
    fpioa::set_function(io::RGB_LED_G, function::GPIO1);
    fpioa::set_function(io::RGB_LED_B, function::GPIO2);
}

fn gpio_init() {
    unsafe {
        let ptr = GPIO::ptr();
        (*ptr).direction.write(|w| {
            w.pin0()
                .variant(direction::OUTPUT)
                .pin1()
                .variant(direction::OUTPUT)
                .pin2()
                .variant(direction::OUTPUT)
        });
    }
}

fn light_up() {
    unsafe {
        let ptr = GPIO::ptr();
        (*ptr)
            .data_output
            .write(|w| w.pin0().clear_bit().pin1().clear_bit().pin2().clear_bit())
    }
}

#[entry]
fn main() -> ! {
    fpioa_init();
    gpio_init();
    light_up();

    loop {}
}
