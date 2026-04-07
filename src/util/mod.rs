//! Shared helper utilities and small constants.

pub mod units;

mod package;
mod patterns;
mod privilege;
mod version;

#[macro_export]
/// Print Debug information using NalaProgress.
macro_rules! dprog {
	($config:expr, $progress:expr, $context:expr, $(,)? $($arg:tt)*) => {
		if $config.debug() {
			let output = std::fmt::format(std::format_args!($($arg)*));
			if $progress.hidden() {
				eprintln!("DEBUG({}): {output}", $context);
			} else {
				$progress.print(&format!("DEBUG({}): {output}", $context))?;
			}
		}
	};
}

pub use package::get_pkg_name;
pub use patterns::{DOMAIN, MIRROR, PACSTALL, UBUNTU_COUNTRY, UBUNTU_URL, URL};
pub(crate) use privilege::get_user;
pub use privilege::sudo_check;
pub use units::{NumSys, UnitStr};
pub use version::version_diff;
