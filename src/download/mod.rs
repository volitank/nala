pub mod downloader;
pub mod proxy;
pub mod uri;

pub use downloader::Downloader;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::LineGauge;
use ratatui::{symbols, Frame};
pub use uri::{Uri, UriFilter};

use crate::config::Theme;
use crate::tui::{paragraph, vblock, Drawable, NalaProgressBar};

impl Drawable for NalaProgressBar<'_> {
	fn draw(&self, f: &mut Frame) {
		let block = vblock(&self.config.color);

		let prog_bar = LineGauge::default()
			.line_set(symbols::line::THICK)
			.ratio(self.ratio())
			.label(self.label().into_line(self.config))
			.filled_style(self.config.color.rat_style(Theme::ProgressFilled))
			.unfilled_style(self.config.color.rat_style(Theme::ProgressUnfilled));

		let [bar_area, percent_area] =
			Layout::horizontal([Constraint::Min(0); 3]).areas(block.inner(f.area()));

		f.render_widget(block, f.area());
		f.render_widget(prog_bar, bar_area);

		let percent = format!(" {:.1}%", self.ratio() * 100.0);
		f.render_widget(
			paragraph(&percent).style(self.config.color.rat_style(Theme::Primary)),
			percent_area,
		);
	}
}
