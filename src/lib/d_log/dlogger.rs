use core::sync::atomic::{AtomicU32, Ordering};

pub struct DLogger;

// Global flag lives OUTSIDE the impl
pub static DLOGGER_HOLD_COUNT: AtomicU32 = AtomicU32::new(0);

impl DLogger {
    #[inline]
    pub fn hold() {
        // increment hold count
        DLOGGER_HOLD_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn release() {
        // decrement, but don't allow underflow
        DLOGGER_HOLD_COUNT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| {
            if x > 0 { Some(x - 1) } else { Some(0) }
        }).ok();
    }

    #[inline]
    pub fn allowed() -> bool {
        // allowed only if counter == 0
        DLOGGER_HOLD_COUNT.load(Ordering::Relaxed) == 0
    }

    #[inline]
    pub fn d_sep() {
        defmt::info!("=======================");
    }
}

#[macro_export]
macro_rules! d_info {
    ($($arg:tt)*) => {
        if $crate::d_log::dlogger::DLogger::allowed() {
            defmt::info!($($arg)*);
        }
    };
}