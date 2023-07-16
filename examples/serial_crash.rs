#![no_std]
#![no_main]

use debug_led::{DebugLed, DebugReportable, reports::*, unwrap_del};

use core::fmt::Write;
use esp32s3_hal::{
    clock::ClockControl,
    gpio::IO,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Rtc,
    Delay,
    PulseControl,
    pulse_control::ClockSource
};
use esp_backtrace as _;
use nb::block;
use esp_hal_smartled::{smartLedAdapter, SmartLedsAdapter};
use esp_println::println;
use smart_leds::{
    brightness,
    gamma,
    RGB8,
    SmartLedsWrite,
};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut timer0 = timer_group0.timer0;
    let mut wdt = timer_group0.wdt;
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);

    // Disable watchdog timer
    wdt.disable();
    rtc.rwdt.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let pulse = PulseControl::new(
      peripherals.RMT,
      &mut system.peripheral_clock_control,
      ClockSource::APB,
      0,
      0,
      0,
    ).unwrap();

    // We use one of the RMT channels to instantiate a `SmartLedsAdapter` which can
    // be used directly with all `smart_led` implementations
    let mut led = <smartLedAdapter!(1)>::new(pulse.channel0, io.pins.gpio48);
    let mut del = DebugLed {
        set_status: &mut |is_set| {
            let col = [if is_set { RGB8::new(255,0,0) } else { RGB8::new(0,0,0) }];
            led.write(brightness(gamma(col.into_iter()), 10)).map_err(|_| ())
        },
        delay: &mut Delay::new(&clocks),
        timings: (250, 50, 50, 250),
        on_report: None,
        debug_print: Some(&|err| {
            println!("\nOops... {err:?}");
        })
    };
    (del.set_status)(false).unwrap();

    unwrap_del!(write!(esp_println::Printer, "Preparing to crash...\n"), &mut del, Binary(0));

    timer0.start(100u32.millis());

    let mut iters: u16 = 30;
    loop {
        let _ = timer0.wait(); // start timer before printing

        unwrap_del!(
            write!(esp_println::Printer, "\rCrashing in {}.{}s", iters / 10, iters % 10),
            &mut del, Binary(0));

        iters = unwrap_del!(iters.checked_sub(1), &mut del, Blink);
        block!(timer0.wait()).unwrap();
    }
}
