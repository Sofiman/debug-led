#![no_std]
#![no_main]

use debug_led::{DebugLed, DebugReportable, DebugReport::*, unwrap_del, expect_del};

use arrayvec::ArrayString;
use core::fmt::Write;
use embedded_graphics::{
    mono_font::ascii::{FONT_6X10, FONT_10X20},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use embedded_graphics::mono_font::MonoTextStyle;
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
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
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
            led.write(brightness(gamma(col.into_iter()), 10))
        },
        delay: &mut Delay::new(&clocks),
        timings: (250, 50, 50, 250),
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
        io.pins.gpio8,
        io.pins.gpio9,
        1u32.MHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    // Initialize display
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    expect_del!(display.init(), "Failed to initialize ssd1306 over i2c", &mut del, Unary(1));

    // Specify different text styles
    let text_style_big = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

    let title = Text::with_alignment(
        "Crash in",
        display.bounding_box().center() - Point::new(0, 8),
        MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
        Alignment::Center,
    );
    let center = display.bounding_box().center() + Point::new(0, (4 + title.character_style.font.character_size.height) as i32);

    timer0.start(100u32.millis());
    let mut iters = 30u16;
    let mut buf = ArrayString::<8>::new();
    loop {
        let _ = timer0.wait();
        unwrap_del!(display.clear(BinaryColor::Off), &mut del, Binary(0));
        title.draw(&mut display).unwrap();

        buf.clear();
        write!(buf, "{}.{}s", iters / 10, iters % 10).unwrap();
        Text::with_alignment(
            buf.as_str(),
            center,
            text_style_big,
            Alignment::Center,
        )
        .draw(&mut display).unwrap();

        unwrap_del!(display.flush(), &mut del, Binary(0));

        iters = unwrap_del!(iters.checked_sub(1), &mut del, Blink);
        block!(timer0.wait()).unwrap();
    }
}
