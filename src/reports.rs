use crate::{DebugLed, DebugReport};

pub struct Blink;

impl DebugReport for Blink {
    fn try_report_once(&self, del: &mut DebugLed) -> Result<(), ()> {
        let (_, off, s_on, _) = del.timings;

        (del.set_status)(true)?;
        del.delay.delay_ms(s_on);
        (del.set_status)(false)?;
        del.delay.delay_ms(off);
        Ok(())
    }

    fn try_report(&self, del: &mut DebugLed) -> Result<(), ()> {
        let (_, off, s_on, _) = del.timings;

        loop {
            (del.set_status)(true)?;
            del.delay.delay_ms(s_on);
            (del.set_status)(false)?;
            del.delay.delay_ms(off);
        }
    }
}

pub struct Unary(pub u8);

impl DebugReport for Unary {
    fn try_report_once(&self, del: &mut DebugLed) -> Result<(), ()> {
        let (_, off, s_on, _) = del.timings;

        for _ in 0..self.0 {
            (del.set_status)(true)?;
            del.delay.delay_ms(s_on);
            (del.set_status)(false)?;
            del.delay.delay_ms(off);
        }
        Ok(())
    }

    fn try_report(&self, del: &mut DebugLed) -> Result<(), ()> {
        let (rep, off, s_on, _) = del.timings;

        loop {
            for _ in 0..self.0 {
                (del.set_status)(true)?;
                del.delay.delay_ms(s_on);
                (del.set_status)(false)?;
                del.delay.delay_ms(off);
            }

            del.delay.delay_ms(rep);
        }
    }
}

pub struct Binary(pub u16);

impl DebugReport for Binary {

    fn try_report_once(&self, del: &mut DebugLed) -> Result<(), ()> {
        let (_, off, s_on, l_on) = del.timings;

        type N = u16;
        const BITS: usize = 8 * core::mem::size_of::<N>();
        let mut val: N = self.0;

        for _ in 0..BITS {
            (del.set_status)(true)?;
            del.delay.delay_ms(if (val & 1) == 1 { s_on } else { l_on });
            (del.set_status)(false)?;
            del.delay.delay_ms(off);
            val >>= 1;
        }

        Ok(())
    }

    fn try_report(&self, del: &mut DebugLed) -> Result<(), ()> {
        let (rep, off, s_on, l_on) = del.timings;

        type N = u16;
        const BITS: usize = 8 * core::mem::size_of::<N>();

        loop {
            let mut val: N = self.0;

            for _ in 0..BITS {
                (del.set_status)(true)?;
                del.delay.delay_ms(if (val & 1) == 1 { s_on } else { l_on });
                (del.set_status)(false)?;
                del.delay.delay_ms(off);
                val >>= 1;
            }

            del.delay.delay_ms(rep);
        }
    }
}

pub struct SmallBinary(pub u16);

impl DebugReport for SmallBinary {

    fn try_report_once(&self, del: &mut DebugLed) -> Result<(), ()> {
        let (_, off, s_on, l_on) = del.timings;

        let mut val = self.0;

        if val == 0 {
            (del.set_status)(true)?;
            del.delay.delay_ms(l_on);
            (del.set_status)(false)?;
            del.delay.delay_ms(off);
        } else {
            while val > 0 {
                (del.set_status)(true)?;
                del.delay.delay_ms(if (val & 1) == 1 { s_on } else { l_on });
                (del.set_status)(false)?;
                del.delay.delay_ms(off);
                val >>= 1;
            }
        }

        Ok(())
    }

    fn try_report(&self, del: &mut DebugLed) -> Result<(), ()> {
        let (rep, off, s_on, l_on) = del.timings;

        loop {
            let mut val = self.0;

            if val == 0 {
                (del.set_status)(true)?;
                del.delay.delay_ms(l_on);
                (del.set_status)(false)?;
                del.delay.delay_ms(off);
            } else {
                while val > 0 {
                    (del.set_status)(true)?;
                    del.delay.delay_ms(if (val & 1) == 1 { s_on } else { l_on });
                    (del.set_status)(false)?;
                    del.delay.delay_ms(off);
                    val >>= 1;
                }
            }

            del.delay.delay_ms(rep);
        }
    }
}
