use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Padding};
use ratatui::Frame;

pub mod fetch;
pub mod progress;
pub mod style;
pub mod summary;

use crate::config::{Config, Theme};

pub(crate) fn frame_block(config: &Config) -> Block<'_> {
	Block::bordered()
		.border_type(BorderType::Rounded)
		.padding(Padding::horizontal(1))
		.style(style::style(config, Theme::Primary))
}

pub(crate) fn borderless_area(f: &mut Frame, area: Rect, title: &str) -> Rect {
	let block = Block::new().title(title).padding(Padding::horizontal(2));
	let inner = block.inner(area);
	f.render_widget(block, area);
	inner
}
