#[macro_export]
/// Print Debug information if the option is set
macro_rules! dprint {
	($config:expr $(,)?, $($arg: tt)*) => {
		if $config.debug() {
			let string = std::fmt::format(std::format_args!($($arg)*));
			eprintln!("DEBUG: {string}");
		}
	};
}
pub use dprint;
