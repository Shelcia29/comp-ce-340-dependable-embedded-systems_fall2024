#![no_std]
#![no_main]

#[rtic::app(device = esp32c3, dispatchers = [FROM_CPU_INTR0, FROM_CPU_INTR1])]
mod app {
    use esp_hal::{self as _, clock::ClockControl, gpio::{Gpio9, Input, PullUp, Gpio7, Output, PushPull}, peripherals::Peripherals, prelude::*, IO, Delay, Rtc};
    use panic_rtt_target as _; // Use RTT for debugging (real-time transfer)
    use rtt_target::{rprintln, rtt_init_print};

    const MAX_SAMPLES: usize = 3;  // Collect the last 3 samples of button presses
    const DEBOUNCE_MS: u64 = 300;  // Constant duration for the LED ON state

    #[shared]
    struct Shared 
    {
        led: Gpio7<Output<PushPull>>,
        delay: Delay,
        time_diffs: [u64; MAX_SAMPLES],
        rhythm_active: bool,
        blink_interval: u64,
    }

    #[local]
    struct Local 
    {
        button: Gpio9<Input<PullUp>>,
        rtc: Rtc<'static>,
        last_sampled_time: u64, 
        sample_count: usize, 
    }

    #[init] // Initialization of the system, runs at startup
    fn init(_: init::Context) -> (Shared, Local) 
    {
        rtt_init_print!();
        rprintln!("Initializing...");

        let peripherals = Peripherals::take();
        let system = peripherals.SYSTEM.split();
        let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
        let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
        let mut button = io.pins.gpio9.into_pull_up_input(); 
        button.listen(esp_hal::gpio::Event::FallingEdge); 
        let led = io.pins.gpio7.into_push_pull_output();
        let rtc = Rtc::new(peripherals.LPWR);
        let delay = Delay::new(&clocks);

        rprintln!("Initialization complete");

// return shared and local resources
        (
            Shared 
            {
                led,
                delay,
                time_diffs: [0; MAX_SAMPLES],
                rhythm_active: false,
                blink_interval: 500,
            },

            Local 
            {
                button,
                rtc,
                last_sampled_time: 0,
                sample_count: 0,
            },
        )
    }
// Task to handle button presses, triggered by GPIO interrupt
    #[task(binds = GPIO, local = [button, rtc, last_sampled_time, sample_count], shared = [led, delay, time_diffs, rhythm_active, blink_interval])]
    fn button(mut cx: button::Context) 
    {
        let current_time = cx.local.rtc.get_time_ms();
        let time_since_last_sample = current_time - *cx.local.last_sampled_time; // Calculate the time difference since the last button press
// Check if the button press is valid (debounced)
        if time_since_last_sample > DEBOUNCE_MS {
            rprintln!("Button sampled at {} ms", current_time); // Debug print the time of the press

            cx.shared.time_diffs.lock(|time_diffs| {
                time_diffs[*cx.local.sample_count % MAX_SAMPLES] = time_since_last_sample;
            });

            *cx.local.sample_count += 1;
            *cx.local.last_sampled_time = current_time;
// If enough samples are collected (at least 3)
            if *cx.local.sample_count >= MAX_SAMPLES {
                let mut total_time = 0; // Calculate the average interval between button presses

                cx.shared.time_diffs.lock(|time_diffs| {
                    for i in 1..MAX_SAMPLES {  // Start from 1 to avoid the first uninitialized slot
                        total_time += time_diffs[i];
                    }
                });

                let avg_interval = total_time / (MAX_SAMPLES - 1) as u64; // Compute the average interval between samples

                rprintln!("Average blink interval: {} ms", avg_interval); // Debug print the average interval
                
                // Update the blink interval
                cx.shared.blink_interval.lock(|blink_interval| {
                    *blink_interval = avg_interval;
                });
                
                // Start or update the LED rhythm task if it's not already running
                cx.shared.rhythm_active.lock(|rhythm_active| {
                    if !*rhythm_active {
                        *rhythm_active = true;
                        rhythm_blink::spawn().unwrap();  // Start blinking task
                    }
                });
            }
        }
        cx.local.button.clear_interrupt(); // Clear the interrupt flag for the button to prepare for the next interrupt
    }

    #[task(shared = [led, delay, rhythm_active, blink_interval])]
    async fn rhythm_blink(mut cx: rhythm_blink::Context)  // Task to blink the LED based on the recorded rhythm (runs in a loop)
    {
        // Loop to keep blinking until rhythm is no longer active
        loop {
            let rhythm_active = cx.shared.rhythm_active.lock(|ra| *ra);

            if !rhythm_active // If rhythm is no longer active, exit the loop and stop blinking
            {
                break; 
            }

            let blink_interval = cx.shared.blink_interval.lock(|bi| *bi); // Get the current blink interval

            rprintln!("Blinking LED with interval: {} ms", blink_interval); // Blink the LED at the current interval
            
            cx.shared.led.lock(|led| {
                led.set_high().unwrap();   // Turn the LED on
            });
            cx.shared.delay.lock(|delay| {
                delay.delay_ms(500 as u32);  // LED stays on for 500ms
            });
            cx.shared.led.lock(|led| {
                led.set_low().unwrap();  // Turn the LED off
            });
            cx.shared.delay.lock(|delay| {
                delay.delay_ms(blink_interval as u32);  // Delay according to blink interval
            });
        }
    }
}