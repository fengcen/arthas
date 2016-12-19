
#[doc(hidden)]
#[macro_export]
macro_rules! thread_trace {
    ($fmt:tt) => (trace!("{}", format!("{}: {}", ::thread_id::get(), $fmt)));
    ($fmt:tt, $($arg:tt)+) => (trace!("{}", format!("{}: {}", ::thread_id::get(), format!($fmt, $($arg)+))))
}
