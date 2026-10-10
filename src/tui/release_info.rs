use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};
use rust_apt::progress::ReleaseInfoChanges;

use super::style as tui_style;
use crate::config::{Config, Theme};
use crate::t;
use crate::terminal::TerminalGuard;
use crate::util::{confirmation_choice, confirmation_keys, response_answer};

pub(crate) fn confirm(config: &Config, info: &ReleaseInfoChanges) -> Result<bool> {
	let mut terminal = TerminalGuard::new()?;
	let keys = confirmation_keys(&t!("prompt-choice"));

	loop {
		terminal
			.terminal_mut()
			.draw(|frame| render(frame, config, info))?;

		if let Event::Key(key) = event::read()?
			&& key.kind == KeyEventKind::Press
			&& let Some(answer) = key_answer(&key, keys)
		{
			return Ok(answer);
		}
	}
}

pub(crate) fn render(frame: &mut ratatui::Frame, config: &Config, info: &ReleaseInfoChanges) {
	let title = t!("release-info-title");
	let keys = confirmation_keys(&t!("prompt-choice"));
	let block = super::summary::header_block(config, &title)
		.border_style(tui_style::style(config, Theme::Notice));
	let [explanation, details] = Layout::vertical([Constraint::Length(3), Constraint::Min(1)])
		.areas(block.inner(frame.area()));
	let mut content = release_details(config, info);
	content.push_line(Line::default());
	content.push_line(
		Line::styled(
			format!(
				"{} {}",
				t!("release-info-confirm"),
				confirmation_choice(keys, false)
			),
			tui_style::style(config, Theme::Highlight),
		)
		.centered(),
	);
	content.push_line(
		Line::styled(
			t!("release-info-help", "yes" => keys.0.to_string(), "no" => keys.1.to_string()),
			tui_style::style(config, Theme::Secondary),
		)
		.centered(),
	);

	frame.render_widget(block, frame.area());
	frame.render_widget(
		Paragraph::new(t!("release-info-explanation"))
			.centered()
			.wrap(Wrap::default())
			.style(tui_style::style(config, Theme::Regular)),
		explanation,
	);
	frame.render_widget(Paragraph::new(content).wrap(Wrap { trim: false }), details);
}

fn release_details(config: &Config, info: &ReleaseInfoChanges) -> Text<'static> {
	let label = tui_style::style(config, Theme::Primary);
	let regular = tui_style::style(config, Theme::Regular);
	let old = tui_style::style(config, Theme::Secondary);
	let new = tui_style::style(config, Theme::Highlight);
	let mut lines = vec![
		Line::from(vec![
			Span::styled(format!("{}: ", t!("release-info-repository")), label),
			Span::styled(info.uri.clone(), regular),
		]),
		Line::from(vec![
			Span::styled(format!("{}: ", t!("release-info-distribution")), label),
			Span::styled(info.dist.clone(), regular),
		]),
		Line::default(),
		Line::styled(t!("release-info-changes"), label),
	];

	for change in &info.changes {
		if change.old_value.is_empty() && change.new_value.is_empty() {
			lines.push(Line::styled(format!("  {}", change.message), regular));
		} else {
			lines.push(Line::from(vec![
				Span::styled(format!("  {}: ", change.field), label),
				Span::styled(change.old_value.clone(), old),
				Span::styled(" → ", regular),
				Span::styled(change.new_value.clone(), new),
			]));
		}
	}

	Text::from(lines)
}

fn key_answer(key: &KeyEvent, keys: (char, char)) -> Option<bool> {
	if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c' | 'C'))
	{
		return Some(false);
	}

	match key.code {
		KeyCode::Char('q' | 'Q') | KeyCode::Enter | KeyCode::Esc => Some(false),
		KeyCode::Char(answer) => response_answer(&answer.to_string(), false, keys),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

	use super::key_answer;

	#[test]
	fn release_info_prompt_defaults_to_rejection() {
		let key_answer = |key: &KeyEvent| super::key_answer(key, ('y', 'n'));
		assert_eq!(
			key_answer(&KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
			Some(false)
		);
		assert_eq!(
			key_answer(&KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)),
			Some(true)
		);
		assert_eq!(
			key_answer(&KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
			Some(false)
		);
		assert_eq!(
			key_answer(&KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
			Some(false)
		);
		assert_eq!(
			key_answer(&KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
			None
		);
	}

	#[test]
	fn release_info_prompt_accepts_translated_keys() {
		for (yes, no) in [('j', 'n'), ('д', 'н'), ('e', 'h'), ('s', 'n')] {
			for answer in [yes, yes.to_uppercase().next().unwrap(), 'y'] {
				assert_eq!(
					key_answer(
						&KeyEvent::new(KeyCode::Char(answer), KeyModifiers::NONE),
						(yes, no)
					),
					Some(true)
				);
			}
			for answer in [no, no.to_uppercase().next().unwrap(), 'n'] {
				assert_eq!(
					key_answer(
						&KeyEvent::new(KeyCode::Char(answer), KeyModifiers::NONE),
						(yes, no)
					),
					Some(false)
				);
			}
			assert_eq!(
				key_answer(
					&KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
					(yes, no)
				),
				Some(false)
			);
		}
	}
}
