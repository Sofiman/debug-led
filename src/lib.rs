#![no_std]

use core::fmt::Debug;
use core::ops::FnMut;
use embedded_hal::blocking::delay::DelayMs;

pub mod reports;

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
    fn unwrap_del<DR: DebugReport>(self, dispatcher: &mut DebugLed, report: DR) -> T;
    fn expect_del<DR: DebugReport>(self, msg: &str, dispatcher: &mut DebugLed, report: DR) -> T;
}

impl<Res, Error: Debug> DebugReportable<Res> for Result<Res, Error> {
    fn unwrap_del<DR: DebugReport>(self, dispatcher: &mut DebugLed, report: DR) -> Res {
        if let Ok(val) = self {
            return val;
        }
        if let Some(print) = &dispatcher.debug_print {
            if let Err(err) = &self {
                print(err);
            }
        }
        report.report(dispatcher);
        self.unwrap()
    }

    fn expect_del<DR: DebugReport>(self, msg: &str, dispatcher: &mut DebugLed, report: DR) -> Res {
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
        report.report(dispatcher);
        self.expect(msg)
    }
}

impl<Res> DebugReportable<Res> for Option<Res> {
    fn unwrap_del<DR: DebugReport>(self, dispatcher: &mut DebugLed, report: DR) -> Res {
        if let Some(val) = self {
            return val;
        }
        if let Some(print) = &dispatcher.debug_print {
            print(&"called `Option::unwrap()` on a `None` value");
        }
        report.report(dispatcher);
        self.unwrap()
    }

    fn expect_del<DR: DebugReport>(self, msg: &str, dispatcher: &mut DebugLed, report: DR) -> Res {
        if let Some(val) = self {
            return val;
        }
        if let Some(print) = &dispatcher.debug_print {
            print(&msg);
        }
        report.report(dispatcher);
        self.expect(msg)
    }
}

pub trait DebugReport {
    fn try_report_once(&self, del: &mut DebugLed) -> Result<(), ()>;

    fn report_once(&self, del: &mut DebugLed) {
        Self::try_report_once(self, del)
            .expect("Failed to report error using debug-led");
    }

    fn try_report(&self, del: &mut DebugLed) -> Result<(), ()>;

    fn report(&self, del: &mut DebugLed) {
        Self::try_report(self, del)
            .expect("Failed to report error using debug-led");
    }
}

pub const DEFAULT_TIMINGS: (u32, u32, u32, u32) = (1000, 500, 500, 1000);

pub struct DebugLed<'a, 'b, 'c> {
    /// Timing fields are (repetition, off, short_on, long_on)
    pub timings: (u32, u32, u32, u32),
    pub debug_print: Option<&'c dyn Fn(&dyn Debug)>,
    pub on_report: Option<&'c mut dyn FnMut(&dyn DebugReport)>,
    pub set_status: &'a mut dyn FnMut(bool) -> Result<(), ()>,
    pub delay: &'b mut dyn DelayMs<u32>
}
