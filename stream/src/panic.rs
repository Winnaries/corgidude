use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};

/** Send panic messages to UARTHS at 115200 baud */
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        // add some side effect to prevent this from turning into a UDF instruction
        // see rust-lang/rust#28728 for details
        atomic::compiler_fence(Ordering::SeqCst)
    }
}
