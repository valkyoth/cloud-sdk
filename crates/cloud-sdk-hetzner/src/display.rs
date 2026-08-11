/// Rejects characters that can alter, hide, or reorder operator-facing text.
pub(crate) fn is_unsafe_display_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200b}'..='\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'..='\u{2069}'
                | '\u{feff}'
        )
}

#[cfg(test)]
mod tests {
    use super::is_unsafe_display_character;

    #[test]
    fn rejects_controls_and_directional_formatting() {
        for character in [
            '\n', '\u{0085}', '\u{061c}', '\u{200b}', '\u{202e}', '\u{2069}', '\u{feff}',
        ] {
            assert!(is_unsafe_display_character(character));
        }
        for character in ['a', ' ', '-', '\u{00e5}'] {
            assert!(!is_unsafe_display_character(character));
        }
    }
}
