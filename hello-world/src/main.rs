#![cfg_attr(any(not(test), target_os = "none"), no_std)]
#![no_main]

#[cfg(any(not(test), target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg_attr(test, allow(unused))]
mod app {
    use core::fmt::Write;
    use l4_ipc::extension::Log;

    pub fn main() {
        let mut console = l4_ipc::base::LOG.console();
        let _ = writeln!(console, "hello rusty console!");
    }
}

/// Nativer Einstiegspunkt ohne C-Runtime
/// Der l4f-Prozesslader (moe) uebergibt die L4Re-Initial-Environment im ersten Register.
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    app::main();
    loop {
        core::hint::spin_loop();
    }
}
