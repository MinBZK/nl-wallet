use std::str::FromStr;

use derive_more::Display;
use derive_more::Into;
use rand::distributions::Distribution;
use rand::distributions::Uniform;

#[derive(Debug, thiserror::Error)]
pub enum ReadableIdentifierParseError {
    #[error("provided string does not contain enough base32 characters")]
    InsufficientCharacters,
    #[error("provided string contains too many base32 characters")]
    ExcessCharacters,
    #[error("invalid character: {0}")]
    InvalidCharacter(char),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Into)]
pub struct ReadableIdentifier<const LEN: usize>(String);

impl<const LEN: usize> ReadableIdentifier<LEN> {
    pub fn new_random() -> Self {
        // Generate LEN u32 values from 0 to 32.
        let distribution = Uniform::from(0..32);
        let mut rng = rand::thread_rng();

        let mut identifier_string = String::with_capacity(LEN);

        for _index in 0..LEN {
            // Do some simple arithmetic to convert this number to the corresponding
            // Unicode codepoint according to Crockford base32.
            let codepoint = match distribution.sample(&mut rng) {
                // Numbers 0 through 9.
                value if (0..=9).contains(&value) => value + 48,
                // Letters A through H.
                value if (10..=17).contains(&value) => value + 55,
                // Letters J and K.
                value if (18..=19).contains(&value) => value + 56,
                // Letters M and N.
                value if (20..=21).contains(&value) => value + 57,
                // Letters P through T.
                value if (22..=26).contains(&value) => value + 58,
                // Letters V through Z.
                value if (27..=31).contains(&value) => value + 59,
                _ => unreachable!("random number should be between 0 and 31"),
            };

            identifier_string
                .push(char::from_u32(codepoint).expect("random number should result in a valid Unicode codepoint"));
        }

        Self(identifier_string)
    }

    fn try_parse(source: &str) -> Result<Self, ReadableIdentifierParseError> {
        if source.len() < LEN {
            return Err(ReadableIdentifierParseError::InsufficientCharacters);
        }

        let mut identifier_string = String::with_capacity(LEN);

        for source_char in source.chars() {
            let output_char = match source_char.to_ascii_uppercase() {
                // Any dashes should be ignored.
                '-' => None,
                // Both the letter I and L should be interpreted as a 1.
                'I' | 'L' => Some('1'),
                // The letter O should be interprested as a 0.
                'O' => Some('0'),
                // All other letters except U are allowed.
                char if char.is_ascii_alphanumeric() && char != 'U' => Some(char),
                char => return Err(ReadableIdentifierParseError::InvalidCharacter(char)),
            };

            if output_char.is_some() && identifier_string.len() >= LEN {
                return Err(ReadableIdentifierParseError::ExcessCharacters);
            }

            if let Some(output_char) = output_char {
                identifier_string.push(output_char);
            }
        }

        if identifier_string.len() < LEN {
            return Err(ReadableIdentifierParseError::InsufficientCharacters);
        }

        Ok(Self(identifier_string))
    }
}

impl<const LEN: usize> AsRef<str> for ReadableIdentifier<LEN> {
    fn as_ref(&self) -> &str {
        let Self(inner) = self;

        inner
    }
}

impl<const LEN: usize> FromStr for ReadableIdentifier<LEN> {
    type Err = ReadableIdentifierParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_parse(s)
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use rstest::rstest;

    use super::ReadableIdentifier;
    use super::ReadableIdentifierParseError;

    #[test]
    fn test_new_random() {
        let identifier = ReadableIdentifier::<32>::new_random();
        let identifier_string = identifier.to_string();

        assert_eq!(identifier_string.len(), 32);

        let parsed_identifier = identifier_string
            .parse()
            .expect("randomly generated identifier should parse successfully");

        assert_eq!(identifier, parsed_identifier);

        let another_identifier = ReadableIdentifier::<32>::new_random();

        assert_ne!(identifier, another_identifier);
    }

    #[test]
    fn test_useless_type() {
        let _ = ReadableIdentifier::<0>::new_random()
            .to_string()
            .parse::<ReadableIdentifier<0>>()
            .expect("useless identifier with 0 length should parse from empty string");
    }

    #[rstest]
    #[case("ABCDEF012345QRSXYZ", "ABCDEF012345QRSXYZ")]
    #[case("ABCDEF-012345-QRSXYZ", "ABCDEF012345QRSXYZ")]
    #[case("ABC-DEF-012-345-QRS-XYZ", "ABCDEF012345QRSXYZ")]
    #[case("---AB-C-DEF012---34----5-Q---RS-XYZ-", "ABCDEF012345QRSXYZ")]
    #[case("AbcdEFoL2345QRsXyZ", "ABCDEF012345QRSXYZ")]
    #[case("abcdef-012345-qrsxyz", "ABCDEF012345QRSXYZ")]
    fn test_parse(#[case] input: &str, #[case] expected: &str) {
        let identifier = input
            .parse::<ReadableIdentifier<18>>()
            .expect("parsing identifier string should succeed");

        assert_eq!(identifier.to_string(), expected);
    }

    #[rstest]
    #[case("")]
    #[case("123")]
    #[case("--1--2--3--")]
    fn test_parse_insufficient_characters(#[case] input: &str) {
        let error = input
            .parse::<ReadableIdentifier<4>>()
            .expect_err("parsing identifier string should not succeed");

        assert_matches!(error, ReadableIdentifierParseError::InsufficientCharacters);
    }

    #[rstest]
    #[case("12345")]
    #[case("ThisIdentifierIsWayTooLong")]
    fn test_parse_excess_characters(#[case] input: &str) {
        let error = input
            .parse::<ReadableIdentifier<4>>()
            .expect_err("parsing identifier string should not succeed");

        assert_matches!(error, ReadableIdentifierParseError::ExcessCharacters);
    }

    #[rstest]
    #[case("RUST", 'U')]
    #[case("huh!", 'U')]
    #[case("hah!", '!')]
    #[case("0Ø00", 'Ø')]
    #[case("t💩rd", '💩')]
    #[case("迷惑です", "迷")]
    fn test_parse_invalid_character(#[case] input: &str, #[case] invalid_char: char) {
        let error = input
            .parse::<ReadableIdentifier<4>>()
            .expect_err("parsing identifier string should not succeed");

        assert_matches!(error, ReadableIdentifierParseError::InvalidCharacter(char) if char == invalid_char);
    }
}
