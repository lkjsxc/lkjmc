#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn decide(valid_request: i32) -> i32 {
    if valid_request == 1 { 1 } else { 0 }
}
