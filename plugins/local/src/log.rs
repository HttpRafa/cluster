#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::cloudlet::driver::log::log_string($crate::cloudlet::driver::log::Level::Debug, format!($($arg)*).as_str())
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::cloudlet::driver::log::log_string($crate::cloudlet::driver::log::Level::Info, format!($($arg)*).as_str())
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::cloudlet::driver::log::log_string($crate::cloudlet::driver::log::Level::Warn, format!($($arg)*).as_str())
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::cloudlet::driver::log::log_string($crate::cloudlet::driver::log::Level::Error, format!($($arg)*).as_str())
    };
}
