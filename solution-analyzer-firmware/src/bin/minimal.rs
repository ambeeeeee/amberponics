#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use test_app as _; // global logger + panicking-behavior + memory layout

// TODO(7) Configure the `rtic::app` macro
#[rtic::app(
    // TODO: Replace `some_hal::pac` with the path to the PAC
    device = stm32h7xx_hal::pac,
    peripherals = true,
    dispatchers = [UART9]
)]
mod app {
    use stm32h7xx_hal::prelude::*;

    // =================================================================================
    //                             Shared Resources
    // =================================================================================
    #[shared]
    struct Shared {}

    // =================================================================================
    //                               Local Resources
    // =================================================================================
    #[local]
    struct Local {}

    // =================================================================================
    //                           Firmware Initialization
    // =================================================================================
    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        (Shared {}, Local {})
    }
}
