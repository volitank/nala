use std::time::{Duration, Instant};

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::style::ContentStyle;
use ratatui::layout::Rect;
use ratatui::prelude::IntoCrossterm;
use ratatui::widgets::Widget;
use rust_apt::progress::{ReleaseInfoChange, ReleaseInfoChanges};

use super::{MirrorProgress, ProgressMessage, ProgressState, ProgressView};
use crate::config::Config;
use crate::tui::{fetch, release_info};

const MIB: u64 = 1024 * 1024;

fn print_buffer(name: &str, buf: &Buffer) {
	println!("\n=== {name} ({}x{}) ===", buf.area.width, buf.area.height);
	for y in 0..buf.area.height {
		let end = (0..buf.area.width)
			.rfind(|&x| buf.cell((x, y)).unwrap().symbol() != " ")
			.map_or(0, |x| x + 1);
		let mut current = ContentStyle::default();
		let mut run = String::new();
		for x in 0..end {
			let cell = buf.cell((x, y)).unwrap();
			let style: ContentStyle = cell.style().into_crossterm();
			if style != current && !run.is_empty() {
				print!("{}", current.apply(&run));
				run.clear();
			}
			current = style;
			run.push_str(cell.symbol());
		}
		if run.is_empty() {
			println!();
		} else {
			println!("{}", current.apply(run));
		}
	}
}

fn print_progress(name: &str, width: u16, state: &ProgressState) {
	let area = Rect::new(0, 0, width, state.view.viewport_lines());
	let mut buf = Buffer::empty(area);
	crate::tui::progress::render_progress_view(&mut buf, area, &Config::default(), state);
	print_buffer(name, &buf);
}

fn transfer_state(view: ProgressView) -> ProgressState {
	let mut state = ProgressState::new(view);
	state.length = 33 * MIB;
	state.position = 19 * MIB;
	state.transferred = state.position;
	state.started = Instant::now() - Duration::from_millis(3500);
	state
}

fn download_state(mirrors: Vec<MirrorProgress>) -> ProgressState {
	let mut state = transfer_state(ProgressView::Download {
		mirrors: mirrors.len(),
	});
	state.set_items(43, 80);
	state.set_message(ProgressMessage::new(
		"Last completed: ",
		vec!["libssl3t64_3.5.1-1_arm64.deb".into()],
	));
	state.set_mirrors(mirrors);
	state
}

fn mirror(name: &str, active: usize, rate_mib: Option<u64>, seen: bool) -> MirrorProgress {
	MirrorProgress::new(
		name.into(),
		active,
		3,
		rate_mib.map(|rate| rate * MIB),
		seen,
	)
}

fn print_release_info(config: &Config) {
	let info = ReleaseInfoChanges {
		uri: "https://deb.example.org/debian".into(),
		dist: "stable".into(),
		changes: vec![
			ReleaseInfoChange {
				field: "Origin".into(),
				old_value: "Example Linux".into(),
				new_value: "Example GNU/Linux".into(),
				message: String::new(),
				default_action: false,
			},
			ReleaseInfoChange {
				field: "Version".into(),
				old_value: "12".into(),
				new_value: "13".into(),
				message: String::new(),
				default_action: false,
			},
		],
	};
	let backend = TestBackend::new(80, 24);
	let mut terminal = Terminal::new(backend).unwrap();
	terminal
		.draw(|frame| release_info::render(frame, config, &info))
		.unwrap();
	print_buffer(
		"Repository information changed",
		terminal.backend().buffer(),
	);
}

fn print_fetch(config: &Config) {
	let area = Rect::new(0, 0, 80, 24);
	let mut buf = Buffer::empty(area);
	let mut app = fetch::App::new(
		config,
		vec![
			("https://deb.debian.org/debian".into(), 18),
			("https://mirror.example.net/debian".into(), 31),
			("https://debian.osuosl.org/debian".into(), 47),
			("https://slow.example.org/debian".into(), 112),
		],
	);
	Widget::render(&mut app, area, &mut buf);
	print_buffer("Fetch mirror selection", &buf);
}

#[test]
#[ignore = "manual TUI gallery"]
fn tui_preview() {
	let config = Config::default();

	let one_mirror = download_state(vec![mirror("deb.debian.org", 3, Some(12), true)]);
	print_progress("Download - one mirror", 80, &one_mirror);

	let many_mirrors = download_state(vec![
		mirror("deb.debian.org", 3, Some(13), true),
		mirror("deb.volian.org", 0, Some(7), true),
		mirror("mirror.example.net", 1, None, false),
		mirror("fallback.example.org", 2, Some(4), true),
	]);
	print_progress("Download - mirror overflow", 80, &many_mirrors);
	print_progress("Download - narrow", 60, &many_mirrors);

	let mut update = transfer_state(ProgressView::Update);
	update.set_message(ProgressMessage::new(
		"Updated: ",
		vec!["deb.debian.org/debian stable InRelease".into()],
	));
	print_progress("Update", 80, &update);

	let mut install = ProgressState::new(ProgressView::Install);
	install.set_length(100);
	install.set_position(43);
	install.started = Instant::now() - Duration::from_millis(7500);
	install.set_item("libssl3t64:arm64".into());
	install.set_message(ProgressMessage::new(
		"Status: ",
		vec!["Configuring libssl3t64:arm64".into()],
	));
	print_progress("Install", 80, &install);

	let mut score = ProgressState::new(ProgressView::MirrorScore);
	score.set_length(12);
	score.set_position(7);
	score.started = Instant::now() - Duration::from_millis(3500);
	score.set_message(ProgressMessage::new(
		"Testing: ",
		vec!["mirror.example.net".into()],
	));
	print_progress("Mirror scoring", 80, &score);

	print_release_info(&config);
	print_fetch(&config);
}
