// Git's remote helper protocol uses stderr as the user-facing output.
// This macro prints to stderr with a "remote:" prefix.
// It also prints to the log with a "[user-facing]" prefix.
#[macro_export]
macro_rules! print_user {
    ($($arg:tt)*) => {
        let msg = format!($($arg)*);
        log::info!("[user-facing] remote: {}", msg);
        eprintln!("remote: {}", msg);
    };
}
