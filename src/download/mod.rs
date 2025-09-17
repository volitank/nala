pub mod downloader;
pub mod proxy;
pub mod uri;

pub use downloader::Downloader;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::LineGauge;
use ratatui::{symbols, Frame};
pub use uri::{Uri, UriFilter};

use crate::config::Theme;
use crate::tui::{paragraph, vblock, Drawable, NalaProgressBar};

impl Drawable for NalaProgressBar<'_> {
	fn draw(&self, f: &mut Frame, area: Rect) {
		// let block = vblock(&self.config.color);
		let prog_bar = self.bar();

		// let [bar_area, percent_area] = self.constraints(&block, block.inner(area));
		let [bar_area, percent_area] =
			Layout::horizontal([Constraint::Fill(100), Constraint::Min(6)]).areas(area);
		// f.render_widget(block, area);
		f.render_widget(prog_bar, bar_area);

		let percent = format!(" {:.1}%", self.ratio() * 100.0);
		f.render_widget(
			paragraph(&percent).style(self.config.color.rat_style(Theme::Primary)),
			percent_area,
		);
	}
}
