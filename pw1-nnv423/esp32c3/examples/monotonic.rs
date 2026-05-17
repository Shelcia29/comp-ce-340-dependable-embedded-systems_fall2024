//! examples/async-delay.rs

#![no_main]
#![no_std]

use panic_rtt_target as _;
#[rtic::app(device = esp32c3, dispatchers = [])]
mod app {
    use esp_hal as _;
    use rtic_monotonics::esp32c3::prelude::*;
    esp32c3_systimer_monotonic!(Mono);
    use rtt_target::{rprintln, rtt_init_print};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        rtt_init_print!();
        rprintln!("init");

        let timer = cx.device.SYSTIMER;

        Mono::start(timer);

        foo::spawn().unwrap();
        bar::spawn().unwrap();
        baz::spawn().unwrap();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_cx: foo::Context) {
        rprintln!("hello from foo");
        Mono::delay(2_u64.secs()).await;
        rprintln!("bye from foo");
    }

    #[task]
    async fn bar(_cx: bar::Context) {
        rprintln!("hello from bar");
        Mono::delay(3_u64.secs()).await;
        rprintln!("bye from bar");
    }

    #[task]
    async fn baz(_cx: baz::Context) {
        rprintln!("hello from baz");
        Mono::delay(4_u64.secs()).await;
        rprintln!("bye from baz");
    }
}
