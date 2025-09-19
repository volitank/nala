pub mod downloader;
pub mod proxy;
pub mod uri;

pub use downloader::Downloader;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::Frame;
pub use uri::{Uri, UriFilter};

use crate::config::Theme;
use crate::tui::progress::DisplayGroup;
use crate::tui::{paragraph, borderless_area, Drawable, NalaProgressBar};



impl Drawable for NalaProgressBar<'_> {
	fn draw(&self, f: &mut Frame, area: Rect) {
		// let inner = borderless_area(f, area, "Progress:");
		let [info_area, progress] = ratatui::layout::Layout::default()
			.direction(ratatui::layout::Direction::Vertical)
			.constraints([
				ratatui::layout::Constraint::Length(3),
				ratatui::layout::Constraint::Length(1),
			])
			.areas(area);

		let mut dg = DisplayGroup::new_str(self.config, "Progress:");
		dg.insert("Total:".to_string(), self.current_total());
		dg.insert("Speed:".to_string(), format!("{}/s", self.unit.str(self.per_sec() as u64)));


		let prog_bar = self.bar();
		let percent = format!(" {:.1}%", self.ratio() * 100.0);

		let [bar_area, percent_area, _buffer] = Layout::horizontal([
			Constraint::Max(32),
			Constraint::Length(percent.len() as u16 + 1),
			Constraint::Min(0),
		])
		.flex(Flex::Legacy)
		.areas(progress);

		dg.draw(f, info_area);
		f.render_widget(prog_bar, bar_area);
		f.render_widget(
			paragraph(&percent).style(self.config.color.rat_style(Theme::Regular)),
			percent_area,
		);
	}
}
