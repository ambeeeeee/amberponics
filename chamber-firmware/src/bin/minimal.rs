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

    use stm32h7xx_hal::device::I2C1;
    use stm32h7xx_hal::i2c::I2c;
    use stm32h7xx_hal::pac::Peripherals;

    use stm32h7xx_hal::spi::Event;

    use stm32h7xx_hal::{prelude::*, spi};
    use test_app::sensors::{HumiditySensor, OxygenSensor, Sensors};

    // =================================================================================
    //                             Shared Resources
    // =================================================================================
    #[shared]
    struct Shared {
        i2c_atlas: I2c<I2C1>,
        atlas_command: (),
        atlas_response: (),
    }

    // =================================================================================
    //                               Local Resources
    // =================================================================================
    #[local]
    struct Local {
        atlas_sensors: Sensors<2>,
    }

    // =================================================================================
    //                           Firmware Initialization
    // =================================================================================
    #[init(local = [
        // Atlas Scientific Sensors
        humidity_sensor: HumiditySensor = HumiditySensor::new(),
        oxygen_sensor: OxygenSensor = OxygenSensor::new()
    ])]
    fn init(cx: init::Context) -> (Shared, Local) {
        defmt::info!("init");
        let dp = Peripherals::take().unwrap();

        let power = dp.PWR.constrain();
        let power_config = power.freeze();

        let rcc = dp.RCC.constrain();
        let ccdr = rcc
            .sys_ck(96.MHz())
            .pll1_q_ck(48.MHz())
            .freeze(power_config, &dp.SYSCFG);
        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);

        // Configure I2C
        let scl = gpiob.pb8.into_alternate_open_drain();
        let sda = gpiob.pb9.into_alternate_open_drain();

        let i2c = dp
            .I2C1
            .i2c((scl, sda), 100.kHz(), ccdr.peripheral.I2C1, &ccdr.clocks);

        // Create atlas scientific sensors processor.
        let atlas_sensors = Sensors {
            sensors: [cx.local.humidity_sensor as _, cx.local.oxygen_sensor as _],
            current_operation: None,
        };

        // TODO setup monotonic if used
        // let sysclk = { /* clock setup + returning sysclk as an u32 */ };
        // let token = rtic_monotonics::create_systick_token!();
        // rtic_monotonics::systick::Systick::new(cx.core.SYST, sysclk, token);

        // task1::spawn().ok();

        (
            Shared {
                // Initialization of shared resources go here
                i2c_atlas: i2c,
                atlas_command: (),
                atlas_response: (),
            },
            Local { atlas_sensors },
        )
    }

    // =================================================================================
    //                        Atlas Scientific Sensor Operations
    // =================================================================================
    
    /// Processes current Atlas Scientific sensor operations.
    #[task(local = [atlas_sensors])]
    async fn atlas_sensors(_cx: atlas_sensors::Context) {}

    /// Spawn to send command to Atlas Scientific sensor.
    #[task(shared = [&atlas_command, i2c_atlas], priority = 1)]
    async fn send_atlas_command(_cx: send_atlas_command::Context) {}

    /// Spawn to handle response from target Atlas Scientific sensor. The result is stored
    /// in the shared `atlas_response` value.
    #[task(shared = [atlas_response], priority = 1)]
    async fn handle_atlas_response(_cx: handle_atlas_response::Context) {}

    // =================================================================================
    //                         XBEE Operation and Communication
    // =================================================================================
    #[task]
    async fn xbee_handler(_cx: xbee_handler::Context) {}

    #[task]
    async fn xbee_send(_cx: xbee_send::Context) {}

    #[task]
    async fn xbee_recv(_cx: xbee_recv::Context) {}

    // =================================================================================
    //                      Device Self-Check and Health Monitoring
    // =================================================================================
    #[task]
    async fn self_test(_cx: self_test::Context) {}

    #[task]
    async fn validate_setup(_cx: validate_setup::Context) {}
}
