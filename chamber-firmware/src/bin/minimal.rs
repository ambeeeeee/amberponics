#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use chamber_firmware as _; // global logger + panicking-behavior + memory layout

// TODO(7) Configure the `rtic::app` macro
#[rtic::app(
    // TODO: Replace `some_hal::pac` with the path to the PAC
    device = stm32h7xx_hal::pac,
    peripherals = true,
    dispatchers = [UART9]
)]
mod app {

    use defmt::debug;
    use heapless::Vec;
    use rtic_monotonics::systick::fugit::Duration;
    use rtic_monotonics::systick::Systick;
    use stm32h7xx_hal::device::I2C1;
    use stm32h7xx_hal::i2c::I2c;
    use stm32h7xx_hal::pac::Peripherals;

    use rtic_monotonics::Monotonic;
    use stm32h7xx_hal::prelude::*;

    use chamber_firmware::atlas::{
        AtlasCommand, HumiditySensor, OxygenSensor, PendingAction, PendingOperation, ResponseCode,
    };
    use chamber_firmware::sensors::AtlasScientificSensors;

    // =================================================================================
    //                             Shared Resources
    // =================================================================================
    #[shared]
    struct Shared {
        i2c_atlas: I2c<I2C1>,
        atlas_response: Option<Vec<u8, 64>>,
    }

    // =================================================================================
    //                               Local Resources
    // =================================================================================
    #[local]
    struct Local {
        atlas_sensors: AtlasScientificSensors<2>,
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
        let atlas_sensors = AtlasScientificSensors {
            sensors: [cx.local.humidity_sensor as _, cx.local.oxygen_sensor as _],
            current_operation: None,
        };

        // TODO setup monotonic if used
        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 12_000_000, systick_token);

        atlas_sensors::spawn().unwrap();

        (
            Shared {
                // Initialization of shared resources go here
                i2c_atlas: i2c,
                atlas_response: None,
            },
            Local { atlas_sensors },
        )
    }

    // =================================================================================
    //                        Atlas Scientific Sensor Operations
    // =================================================================================

    /// Processes current Atlas Scientific sensor operations.
    #[task(local = [atlas_sensors], shared = [atlas_response])]
    async fn atlas_sensors(mut cx: atlas_sensors::Context) {
        let sensors = cx.local.atlas_sensors;

        if sensors.current_operation.is_none() {
            // Determine the current operation
            let mut soonest_deadline = None;
            let mut soonest = None;
            let mut soonest_index = None;

            for (index, sensor) in sensors.sensors.iter().enumerate() {
                match sensor.pending_action() {
                    operation @ PendingAction::Startup { .. } => {
                        // We'll deal with startup commands immediately
                        sensors.current_operation = Some(PendingOperation {
                            sensor: index,
                            operation: *operation,
                        });

                        break;
                    }
                    PendingAction::Sample { deadline } => {
                        if soonest_deadline.is_none() {
                            soonest_deadline = Some(*deadline);
                            soonest = Some(sensor);
                            soonest_index = Some(index);
                        } else if soonest_deadline.unwrap() >= *deadline {
                            soonest_deadline = Some(*deadline);
                            soonest = Some(sensor);
                            soonest_index = Some(index);
                        }
                    }
                    PendingAction::Receive { deadline } => {
                        if soonest_deadline.is_none() {
                            soonest_deadline = Some(*deadline);
                            soonest = Some(sensor);
                            soonest_index = Some(index);
                        } else if soonest_deadline.unwrap() >= *deadline {
                            soonest_deadline = Some(*deadline);
                            soonest = Some(sensor);
                            soonest_index = Some(index);
                        }
                    }
                }
            }

            // If soonest_deadline filtering is needed to set the current operation, we set it now.
            if sensors.current_operation.is_none() {
                if let (Some(soonest), Some(soonest_index)) = (soonest, soonest_index) {
                    sensors.current_operation = Some(PendingOperation {
                        sensor: soonest_index,
                        operation: *soonest.pending_action(),
                    })
                }
            }
        }

        let next_operation;

        if let Some(mut current_operation) = sensors.current_operation.take() {
            let sensor = sensors.sensors.get_mut(current_operation.sensor).unwrap();

            let address = sensor.address();

            let next_action;

            match current_operation.operation {
                PendingAction::Startup { command_index } => {
                    if sensor.setup_commands().len() > command_index {
                        let send_command = sensor.setup_commands()[command_index];

                        let mut command_buff = Vec::new();
                        command_buff[..send_command.len()].copy_from_slice(send_command);

                        defmt::trace!(
                            "[atlas_sensors] Startup command {} for sensor {} issuing.",
                            send_command,
                            address
                        );
                        send_atlas_command::spawn(AtlasCommand {
                            address: address as usize,
                            command: command_buff,
                        })
                        .unwrap();

                        if sensor.setup_commands().len() > command_index + 1 {
                            next_action = Some(PendingAction::Startup {
                                command_index: command_index + 1,
                            })
                        } else {
                            // We're done running startup commands, queue a sample
                            *sensor.pending_action_mut() = PendingAction::Sample {
                                deadline: Systick::now()
                                    + Duration::<u32, 1, 1000>::from_ticks(5000),
                            };

                            next_action = None
                        }
                    } else {
                        next_action = None
                    }
                }
                PendingAction::Sample { deadline } => {
                    let send_command = sensor.sample_command();

                    let mut command_buff = Vec::new();
                    command_buff[..send_command.len()].copy_from_slice(send_command);

                    defmt::trace!(
                        "[atlas_sensors] Sample command {} for sensor {} issuing.",
                        send_command,
                        address
                    );
                    send_atlas_command::spawn(AtlasCommand {
                        address: address as usize,
                        command: command_buff,
                    })
                    .unwrap();

                    next_action = Some(PendingAction::Receive {
                        deadline: deadline + Duration::<u32, 1, 1000>::from_ticks(1000u32),
                    })
                }
                PendingAction::Receive { deadline: _ } => {
                    defmt::trace!("[atlas_sensors] Handling response for sensor {}.", address);
                    handle_atlas_response::spawn(address as _).unwrap();

                    let response = loop {
                        // Wait for response
                        if let Some(response) =
                            cx.shared.atlas_response.lock(|response| response.take())
                        {
                            break response;
                        }
                    };

                    let response_code = ResponseCode::try_from_probe_response(&response).unwrap();

                    let split = response.split(|c| *c == '\r' as u8);

                    for token in split {
                        defmt::trace!("Read token {:?}", core::str::from_utf8(&token));
                    }

                    let mut split = response.split(|c| *c == '\r' as u8);
                    let token = split.next().unwrap();
                    let token = core::str::from_utf8(&token).unwrap();

                    let value: f64 = token.parse().unwrap();

                    debug!(
                        "Read value {} with status {} from sensor {}",
                        value, response_code, address
                    );

                    // Do thing with response

                    *sensor.pending_action_mut() = PendingAction::Sample {
                        deadline: Systick::now() + Duration::<u32, 1, 1000>::from_ticks(5000),
                    };

                    next_action = None
                }
            }

            if let Some(action) = next_action {
                current_operation.operation = action;
                next_operation = Some(current_operation);
                *sensor.pending_action_mut() = action;
            } else {
                next_operation = None;
            }
        } else {
            next_operation = None;
        }

        if let Some(operation) = next_operation {
            sensors.current_operation = Some(operation);
        }
    }

    /// Spawn to send command to Atlas Scientific sensor.
    #[task(shared = [i2c_atlas], priority = 1)]
    async fn send_atlas_command(mut cx: send_atlas_command::Context, command: AtlasCommand) {
        cx.shared.i2c_atlas.lock(|i2c| {
            debug!(
                "[send_atlas_command] Writing i2c command to {}: {:?}",
                command.address,
                &command.command[..command.command.len()]
            );

            i2c.write(
                u8::try_from(command.address).unwrap(),
                &command.command[..command.command.len()],
            )
            .unwrap();
        })
    }

    /// Spawn to handle response from target Atlas Scientific sensor. The result is stored
    /// in the shared `atlas_response` value.
    #[task(shared = [atlas_response, i2c_atlas], priority = 1)]
    async fn handle_atlas_response(mut cx: handle_atlas_response::Context, address: u8) {
        let mut read_buffer: [u8; 64] = [0; 64];
        let mut offset = 0;

        cx.shared.i2c_atlas.lock(|i2c| {
            defmt::trace!("[read_atlas_response] Reading i2c from {}.", address);

            while i2c.read(address, &mut read_buffer[offset..]).is_ok() {
                offset += 1;
            }
        });

        debug!(
            "[read_atlas_response] Read {} bytes from i2c for {}.",
            offset, address
        );
    }

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
