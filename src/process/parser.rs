use std::ffi::OsString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    pub program: OsString,
    pub args: Vec<OsString>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    EmptyInput,
    UnterminatedQuote,
    DanglingEscape,
    UnsupportedOperator(UnsupportedOperator),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsupportedOperator {
    AndIf,
    OrIf,
    Pipe,
    RedirectIn,
    RedirectOut,
    Sequence,
    Backtick,
    CommandSubstitution,
}

pub fn parse_command_line(input: &str) -> Result<ParsedCommand, ParseError> {
    let mut parser = Parser::new(input);
    parser.skip_whitespace();

    if parser.is_done() {
        return Err(ParseError {
            kind: ParseErrorKind::EmptyInput,
            position: input.len(),
        });
    }

    let mut words = Vec::new();
    loop {
        words.push(parser.parse_word()?);
        parser.skip_whitespace();

        if parser.is_done() {
            break;
        }
    }

    let program = OsString::from(words.remove(0));
    let args = words.into_iter().map(OsString::from).collect();
    Ok(ParsedCommand { program, args })
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "parse error at position {}: {}",
            self.position, self.kind
        )
    }
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "command line is empty"),
            Self::UnterminatedQuote => write!(f, "unterminated quote"),
            Self::DanglingEscape => write!(f, "dangling escape"),
            Self::UnsupportedOperator(operator) => {
                write!(f, "unsupported operator `{operator}`")
            }
        }
    }
}

impl std::fmt::Display for UnsupportedOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AndIf => write!(f, "&&"),
            Self::OrIf => write!(f, "||"),
            Self::Pipe => write!(f, "|"),
            Self::RedirectIn => write!(f, "<"),
            Self::RedirectOut => write!(f, ">"),
            Self::Sequence => write!(f, ";"),
            Self::Backtick => write!(f, "`"),
            Self::CommandSubstitution => write!(f, "$("),
        }
    }
}

impl std::error::Error for ParseError {}

struct Parser<'a> {
    input: &'a str,
    cursor: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
    }

    fn is_done(&self) -> bool {
        self.cursor == self.input.len()
    }

    fn skip_whitespace(&mut self) {
        while let Some((_, ch)) = self.peek_char() {
            if !ch.is_whitespace() {
                return;
            }

            self.advance_char();
        }
    }

    fn parse_word(&mut self) -> Result<String, ParseError> {
        let mut output = String::new();
        let mut saw_quoted_part = false;

        while let Some((position, ch)) = self.peek_char() {
            if ch.is_whitespace() {
                break;
            }

            if let Some(operator) = self.unsupported_operator_at_cursor() {
                return Err(ParseError {
                    kind: ParseErrorKind::UnsupportedOperator(operator),
                    position,
                });
            }

            if ch == '"' {
                self.advance_char();
                saw_quoted_part = true;
                self.parse_quoted_part(position, &mut output)?;
                continue;
            }

            self.advance_char();
            output.push(ch);
        }

        if output.is_empty() && !saw_quoted_part {
            return Err(ParseError {
                kind: ParseErrorKind::EmptyInput,
                position: self.cursor,
            });
        }

        Ok(output)
    }

    fn parse_quoted_part(
        &mut self,
        opening_quote: usize,
        output: &mut String,
    ) -> Result<(), ParseError> {
        loop {
            let Some((position, ch)) = self.advance_char() else {
                return Err(ParseError {
                    kind: ParseErrorKind::UnterminatedQuote,
                    position: opening_quote,
                });
            };

            match ch {
                '"' => return Ok(()),
                '\\' => self.parse_quoted_escape(position, output)?,
                _ => output.push(ch),
            }
        }
    }

    fn parse_quoted_escape(
        &mut self,
        escape_position: usize,
        output: &mut String,
    ) -> Result<(), ParseError> {
        let Some((_, escaped)) = self.advance_char() else {
            return Err(ParseError {
                kind: ParseErrorKind::DanglingEscape,
                position: escape_position,
            });
        };

        match escaped {
            '"' | '\\' => output.push(escaped),
            _ => {
                output.push('\\');
                output.push(escaped);
            }
        }

        Ok(())
    }

    fn unsupported_operator_at_cursor(&self) -> Option<UnsupportedOperator> {
        let rest = &self.input[self.cursor..];
        if rest.starts_with("&&") {
            Some(UnsupportedOperator::AndIf)
        } else if rest.starts_with("||") {
            Some(UnsupportedOperator::OrIf)
        } else if rest.starts_with("$(") {
            Some(UnsupportedOperator::CommandSubstitution)
        } else {
            match rest.chars().next()? {
                '|' => Some(UnsupportedOperator::Pipe),
                '<' => Some(UnsupportedOperator::RedirectIn),
                '>' => Some(UnsupportedOperator::RedirectOut),
                ';' => Some(UnsupportedOperator::Sequence),
                '`' => Some(UnsupportedOperator::Backtick),
                _ => None,
            }
        }
    }

    fn peek_char(&self) -> Option<(usize, char)> {
        self.input[self.cursor..]
            .chars()
            .next()
            .map(|ch| (self.cursor, ch))
    }

    fn advance_char(&mut self) -> Option<(usize, char)> {
        let (position, ch) = self.peek_char()?;
        self.cursor += ch.len_utf8();
        Some((position, ch))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ParseError, ParseErrorKind, ParsedCommand, UnsupportedOperator, parse_command_line,
    };

    fn parsed(program: &str, args: &[&str]) -> ParsedCommand {
        ParsedCommand {
            program: program.into(),
            args: args.iter().map(Into::into).collect(),
        }
    }

    fn parse_error(input: &str) -> ParseError {
        parse_command_line(input).expect_err("parse error")
    }

    #[test]
    fn parses_bare_words() {
        assert_eq!(
            parse_command_line("cargo --version").unwrap(),
            parsed("cargo", &["--version"])
        );
    }

    #[test]
    fn parses_quoted_words() {
        assert_eq!(
            parse_command_line("powershell -NoProfile -Command \"Get-Date\"").unwrap(),
            parsed("powershell", &["-NoProfile", "-Command", "Get-Date"])
        );
    }

    #[test]
    fn preserves_empty_quoted_arguments() {
        assert_eq!(
            parse_command_line("tool \"\" tail").unwrap(),
            parsed("tool", &["", "tail"])
        );
    }

    #[test]
    fn preserves_escaped_quotes_inside_quotes() {
        assert_eq!(
            parse_command_line("tool \"say \\\"hello\\\"\"").unwrap(),
            parsed("tool", &["say \"hello\""])
        );
    }

    #[test]
    fn preserves_escaped_backslashes_inside_quotes() {
        assert_eq!(
            parse_command_line("tool \"C:\\\\Program Files\"").unwrap(),
            parsed("tool", &["C:\\Program Files"])
        );
    }

    #[test]
    fn keeps_unescaped_backslashes_inside_quotes_literal() {
        assert_eq!(
            parse_command_line("tool \"C:\\Program Files\"").unwrap(),
            parsed("tool", &["C:\\Program Files"])
        );
    }

    #[test]
    fn ignores_leading_and_trailing_whitespace() {
        assert_eq!(
            parse_command_line(" \t cargo --version  \r\n").unwrap(),
            parsed("cargo", &["--version"])
        );
    }

    #[test]
    fn reports_empty_input() {
        assert_eq!(parse_error(" \t ").kind, ParseErrorKind::EmptyInput);
    }

    #[test]
    fn reports_unterminated_quotes() {
        assert_eq!(
            parse_error("tool \"unterminated"),
            ParseError {
                kind: ParseErrorKind::UnterminatedQuote,
                position: 5,
            }
        );
    }

    #[test]
    fn reports_dangling_escapes() {
        assert_eq!(
            parse_error("tool \"dangling\\"),
            ParseError {
                kind: ParseErrorKind::DanglingEscape,
                position: 14,
            }
        );
    }

    #[test]
    fn rejects_unsupported_operators() {
        let cases = [
            ("tool | more", UnsupportedOperator::Pipe, 5),
            ("tool < input", UnsupportedOperator::RedirectIn, 5),
            ("tool > output", UnsupportedOperator::RedirectOut, 5),
            ("tool && other", UnsupportedOperator::AndIf, 5),
            ("tool || other", UnsupportedOperator::OrIf, 5),
            ("tool ; other", UnsupportedOperator::Sequence, 5),
            ("tool `date`", UnsupportedOperator::Backtick, 5),
            ("tool $(date)", UnsupportedOperator::CommandSubstitution, 5),
        ];

        for (input, operator, position) in cases {
            assert_eq!(
                parse_error(input),
                ParseError {
                    kind: ParseErrorKind::UnsupportedOperator(operator),
                    position,
                }
            );
        }
    }
}
