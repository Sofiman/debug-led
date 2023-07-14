#![no_std]

use core::fmt::Debug;
use core::ops::FnMut;
use embedded_hal::blocking::delay::DelayMs;

#[macro_export]
macro_rules! derr {
    ($dis:expr, $e:expr => $k:expr) => {
        {
            use debug_led::DebugReport::*;
            $e.unwrap_or_else(|_| {
                $dis.report($k);
                $e.unwrap()
            })
        }
    }
}

pub trait DebugReportable<T> {
    fn unwrap_del<E: Debug>(self, dispatcher: &mut DebugLed<E>, report: DebugReport) -> T;
    fn expect_del<E: Debug>(self, msg: &str, dispatcher: &mut DebugLed<E>, report: DebugReport) -> T;
}

impl<Res, Error: Debug> DebugReportable<Res> for Result<Res, Error> {
    fn unwrap_del<E: Debug>(self, dispatcher: &mut DebugLed<E>, report: DebugReport) -> Res {
        if let Ok(val) = self {
            return val;
        }
        dispatcher.report(report);
        self.unwrap()
    }

    fn expect_del<E: Debug>(self, msg: &str, dispatcher: &mut DebugLed<E>, report: DebugReport) -> Res {
        if let Ok(val) = self {
            return val;
        }
        dispatcher.report(report);
        self.expect(msg)
    }
}

impl<Res> DebugReportable<Res> for Option<Res> {
    fn unwrap_del<E: Debug>(self, dispatcher: &mut DebugLed<E>, report: DebugReport) -> Res {
        if let Some(val) = self {
            return val;
        }
        dispatcher.report(report);
        self.unwrap()
    }

    fn expect_del<E: Debug>(self, msg: &str, dispatcher: &mut DebugLed<E>, report: DebugReport) -> Res {
        if let Ok(val) = self {
            return val;
        }
        dispatcher.report(report);
        self.expect(msg)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum DebugReport {
    #[default]
    Blink,
    Unary(u8),
    Binary(u16)
}

pub const DEFAULT_TIMINGS: (u32, u32, u32, u32) = (1000, 500, 500, 1000);

pub struct DebugLed<'a, 'b, E> {
    pub timings: (u32, u32, u32, u32),
    pub set_status: &'a mut dyn FnMut(bool) -> Result<(), E>,
    pub delay: &'b mut dyn DelayMs<u32> 
}

impl<E: Debug> DebugLed<'_, '_, E> {
    pub fn report(&mut self, report: DebugReport) {
        self.try_report(report).expect("Failed to report error using debug-led");
    }

    pub fn try_report(&mut self, report: DebugReport) -> Result<(), E> {
        let (rep, off, s_on, l_on) = self.timings;
        use DebugReport::*;
        match report {
            Blink => {
                loop {
                    (self.set_status)(true)?;
                    self.delay.delay_ms(s_on);
                    (self.set_status)(false)?;
                    self.delay.delay_ms(off);
                }
            },
            Unary(count) => {
                loop {
                    for _ in 0..count {
                        (self.set_status)(true)?;
                        self.delay.delay_ms(s_on);
                        (self.set_status)(false)?;
                        self.delay.delay_ms(off);
                    }

                    self.delay.delay_ms(rep);
                }
            },
            Binary(value) => {
                loop {
                    let mut val = value;

                    if val == 0 {
                        (self.set_status)(true)?;
                        self.delay.delay_ms(l_on);
                        (self.set_status)(false)?;
                        self.delay.delay_ms(off);
                    } else {
                        while val > 0 {
                            (self.set_status)(true)?;
                            self.delay.delay_ms(if (val & 1) == 1 { s_on } else { l_on });
                            (self.set_status)(false)?;
                            self.delay.delay_ms(off);
                            val >>= 1;
                        }
                    }

                    self.delay.delay_ms(rep);
                }
            },
        }
    }
}
