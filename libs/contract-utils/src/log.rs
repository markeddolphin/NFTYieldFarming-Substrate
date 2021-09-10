#[macro_export]
/// Logs an error with ink, with optional prefix
macro_rules! ink_log_err {
    ($value:expr) => {
        match $value {
            Ok(x) => Ok(x),
            Err(x) => {
                ink_env::debug_println(&ink_prelude::format!("{}", x));
                Err(x)
            }
        }
    };

    ($prefix:expr, $value:expr) => {
        match $value {
            Ok(x) => Ok(x),
            Err(x) => {
                ink_env::debug_println(&ink_prelude::format!("{}{}", $prefix, x));
                Err(x)
            }
        }
    };
}
