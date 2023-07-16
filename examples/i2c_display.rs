#![no_std]
#![no_main]

use debug_led::{DebugLed, DebugReportable, reports::*};

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use esp32s3_hal::{
    clock::ClockControl,
    gpio::IO,
    i2c::I2C,
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
use esp_println::println;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use esp_hal_smartled::{smartLedAdapter, SmartLedsAdapter};
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
        timings: debug_led::DEFAULT_TIMINGS,
        on_report: None,
        debug_print: Some(&|err| {
            println!("Oops... {err:?}");
        })
    };
    (del.set_status)(false).unwrap();

    // Create a new peripheral object with the described wiring
    // and standard I2C clock speed
    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio9, // sda
        io.pins.gpio8, // scl
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    // Start timer (5 second interval)
    timer0.start(5u64.secs());

    // Initialize display
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display.init().unwrap_del(&mut del, Unary(1));

    // Specify different text styles
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let text_style_big = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .build();

    loop {
        // Fill display bufffer with a centered text with two lines (and two text
        // styles)
         Text::with_alignment(
            "esp-hal",
            display.bounding_box().center() + Point::new(0, 0),
            text_style_big,
            Alignment::Center,
        ).draw(&mut display).unwrap();

        Text::with_alignment(
            "Chip: ESP32S3",
            display.bounding_box().center() + Point::new(0, 14),
            text_style,
            Alignment::Center,
        )
        .draw(&mut display).unwrap();

        // Write buffer to display
        display.flush().unwrap_del(&mut del, Binary(2));
        // Clear display buffer
        display.clear(BinaryColor::Off).unwrap_del(&mut del, Binary(2));

        // Wait 5 seconds
        block!(timer0.wait()).unwrap();

        // Write single-line centered text "Hello World" to buffer
        Text::with_alignment(
            "Hello World!",
            display.bounding_box().center(),
            text_style_big,
            Alignment::Center,
        )
        .draw(&mut display).unwrap();

        // Write buffer to display
        display.flush().unwrap_del(&mut del, Binary(3));
        // Clear display buffer
        display.clear(BinaryColor::Off).unwrap_del(&mut del, Binary(3));

        // Wait 5 seconds
        block!(timer0.wait()).unwrap();
    }
}
