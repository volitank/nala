use fluent::FluentArgs;

pub fn translate(id: &str, args: Option<&FluentArgs>) -> String {
	shared::translate(id, args, |errors| {
		crate::debug!("Failed to format Fluent message '{id}': {errors:?}");
	})
}

mod shared;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::t;

	#[test]
	fn translate_falls_back_to_message_id_when_missing() {
		assert_eq!(translate("missing-message-id", None), "missing-message-id");
	}

	#[test]
	fn translate_falls_back_to_message_id_when_formatting_fails() {
		assert_eq!(translate("history-cleared", None), "history-cleared");
	}

	#[test]
	fn translate_formats_plural_messages_with_isolating() {
		assert_eq!(
			t!("history-cleared", "count" => 0),
			"Cleared \u{2068}0\u{2069} history entries."
		);
		assert_eq!(
			t!("history-cleared", "count" => 1),
			"Cleared \u{2068}1\u{2069} history entry."
		);
		assert_eq!(
			t!("history-cleared", "count" => 2),
			"Cleared \u{2068}2\u{2069} history entries."
		);
	}
}
