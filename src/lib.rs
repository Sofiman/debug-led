#![no_std]

use core::fmt::Debug;
use core::ops::FnMut;
use embedded_hal::blocking::delay::DelayMs;

#[macro_export]
macro_rules! unwrap_del {
    ($val:expr, $dis:expr, $k:expr) => {
        $val.expect_del(concat!(file!(), ':', line!(), ':', column!()), $dis, $k)
    }
}

#[macro_export]
macro_rules! expect_del {
    ($val:expr, $msg:expr, $dis:expr, $k:expr) => {
        $val.expect_del(concat!($msg, " in ", file!(), ':', line!(), ':', column!()), $dis, $k)
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
        if let Some(print) = &dispatcher.debug_print {
            if let Err(err) = &self {
                print(err);
            }
        }
        dispatcher.report(report);
        self.unwrap()
    }

    fn expect_del<E: Debug>(self, msg: &str, dispatcher: &mut DebugLed<E>, report: DebugReport) -> Res {
        if let Ok(val) = self {
            return val;
        }
        if let Some(print) = &dispatcher.debug_print {
            // TODO: Maybe combine the two print calls?
            print(&msg);
            if let Err(err) = &self {
                print(err);
            }
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
        if let Some(print) = &dispatcher.debug_print {
            print(&"called `Option::unwrap()` on a `None` value");
        }
        dispatcher.report(report);
        self.unwrap()
    }

    fn expect_del<E: Debug>(self, msg: &str, dispatcher: &mut DebugLed<E>, report: DebugReport) -> Res {
        if let Some(val) = self {
            return val;
        }
        if let Some(print) = &dispatcher.debug_print {
            print(&msg);
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

pub struct DebugLed<'a, 'b, 'c, E> {
    /// Timing fields are (repetition, off, short_on, long_on)
    pub timings: (u32, u32, u32, u32),
    pub debug_print: Option<&'c dyn Fn(&dyn Debug)>,
    pub on_report: Option<&'c mut dyn FnMut(&DebugReport)>,
    pub set_status: &'a mut dyn FnMut(bool) -> Result<(), E>,
    pub delay: &'b mut dyn DelayMs<u32>
}

impl<E: Debug> DebugLed<'_, '_, '_, E> {
    pub fn report(&mut self, report: DebugReport) {
        self.try_report(report).expect("Failed to report error using debug-led");
    }

    pub fn try_report(&mut self, report: DebugReport) -> Result<(), E> {
        let (rep, off, s_on, l_on) = self.timings;
        if let Some(on_report) = &mut self.on_report {
            (on_report)(&report);
        }
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

    pub fn report_once(&mut self, report: DebugReport) {
        self.try_report_once(report).expect("Failed to report error using debug-led");
    }

    pub fn try_report_once(&mut self, report: DebugReport) -> Result<(), E> {
        let (_, off, s_on, l_on) = self.timings;
        if let Some(on_report) = &mut self.on_report {
            (on_report)(&report);
        }
        use DebugReport::*;
        match report {
            Blink => {
                (self.set_status)(true)?;
                self.delay.delay_ms(s_on);
                (self.set_status)(false)?;
                self.delay.delay_ms(off);
            },
            Unary(count) => {
                for _ in 0..count {
                    (self.set_status)(true)?;
                    self.delay.delay_ms(s_on);
                    (self.set_status)(false)?;
                    self.delay.delay_ms(off);
                }
            },
            Binary(value) => {
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
            },
        }
        Ok(())
    }
}
