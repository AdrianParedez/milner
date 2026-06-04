use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    pub program: OsString,
    pub args: Vec<OsString>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionPlan {
    Command(CommandSpec),
    Pipeline {
        left: CommandSpec,
        right: CommandSpec,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub command: ParsedCommand,
    pub stdin: InputSpec,
    pub stdout: OutputSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputSpec {
    Inherit,
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputSpec {
    Inherit,
    File { path: PathBuf, append: bool },
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
    MissingRedirectionTarget(RedirectionOperator),
    EmptyPipelineCommand,
    MultiplePipelines,
    DuplicateStdin,
    DuplicateStdout,
    UnexpectedOperator(RedirectionOperator),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsupportedOperator {
    AndIf,
    OrIf,
    Sequence,
    Backtick,
    CommandSubstitution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectionOperator {
    Stdin,
    StdoutTruncate,
    StdoutAppend,
    Pipe,
}

pub fn parse_execution_line(input: &str) -> Result<ExecutionPlan, ParseError> {
    let mut parser = Parser::new(input);
    let tokens = parser.parse_tokens()?;
    build_execution_plan(&tokens)
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
            Self::MissingRedirectionTarget(operator) => {
                write!(f, "missing target after `{operator}`")
            }
            Self::EmptyPipelineCommand => write!(f, "pipeline command must not be empty"),
            Self::MultiplePipelines => write!(f, "only one pipeline is supported"),
            Self::DuplicateStdin => write!(f, "stdin redirection is already set"),
            Self::DuplicateStdout => write!(f, "stdout redirection is already set"),
            Self::UnexpectedOperator(operator) => write!(f, "unexpected operator `{operator}`"),
        }
    }
}

impl std::fmt::Display for UnsupportedOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AndIf => write!(f, "&&"),
            Self::OrIf => write!(f, "||"),
            Self::Sequence => write!(f, ";"),
            Self::Backtick => write!(f, "`"),
            Self::CommandSubstitution => write!(f, "$("),
        }
    }
}

impl std::fmt::Display for RedirectionOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdin => write!(f, "<"),
            Self::StdoutTruncate => write!(f, ">"),
            Self::StdoutAppend => write!(f, ">>"),
            Self::Pipe => write!(f, "|"),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word {
        value: String,
    },
    Operator {
        kind: RedirectionOperator,
        position: usize,
    },
}

fn build_execution_plan(tokens: &[Token]) -> Result<ExecutionPlan, ParseError> {
    if tokens.is_empty() {
        return Err(ParseError {
            kind: ParseErrorKind::EmptyInput,
            position: 0,
        });
    }

    let mut pipe_index = None;
    for (index, token) in tokens.iter().enumerate() {
        let Token::Operator {
            kind: RedirectionOperator::Pipe,
            position,
        } = token
        else {
            continue;
        };

        if pipe_index.is_some() {
            return Err(ParseError {
                kind: ParseErrorKind::MultiplePipelines,
                position: *position,
            });
        }

        pipe_index = Some(index);
    }

    if let Some(index) = pipe_index {
        let left = parse_command_spec(&tokens[..index], 0)?;
        let right = parse_command_spec(&tokens[index + 1..], index + 1)?;
        Ok(ExecutionPlan::Pipeline { left, right })
    } else {
        Ok(ExecutionPlan::Command(parse_command_spec(tokens, 0)?))
    }
}

fn parse_command_spec(tokens: &[Token], token_offset: usize) -> Result<CommandSpec, ParseError> {
    if tokens.is_empty() {
        return Err(ParseError {
            kind: ParseErrorKind::EmptyPipelineCommand,
            position: token_offset,
        });
    }

    let mut words = Vec::new();
    let mut stdin = InputSpec::Inherit;
    let mut stdout = OutputSpec::Inherit;
    let mut index = 0usize;

    while let Some(token) = tokens.get(index) {
        match token {
            Token::Word { value } => {
                words.push(value.clone());
                index += 1;
            }
            Token::Operator { kind, position } => match kind {
                RedirectionOperator::Stdin => {
                    if stdin != InputSpec::Inherit {
                        return Err(ParseError {
                            kind: ParseErrorKind::DuplicateStdin,
                            position: *position,
                        });
                    }

                    let path = redirection_target(tokens, index + 1, *kind, *position)?;
                    stdin = InputSpec::File(path);
                    index += 2;
                }
                RedirectionOperator::StdoutTruncate | RedirectionOperator::StdoutAppend => {
                    if stdout != OutputSpec::Inherit {
                        return Err(ParseError {
                            kind: ParseErrorKind::DuplicateStdout,
                            position: *position,
                        });
                    }

                    let path = redirection_target(tokens, index + 1, *kind, *position)?;
                    stdout = OutputSpec::File {
                        path,
                        append: *kind == RedirectionOperator::StdoutAppend,
                    };
                    index += 2;
                }
                RedirectionOperator::Pipe => {
                    return Err(ParseError {
                        kind: ParseErrorKind::UnexpectedOperator(*kind),
                        position: *position,
                    });
                }
            },
        }
    }

    if words.is_empty() {
        return Err(ParseError {
            kind: ParseErrorKind::EmptyPipelineCommand,
            position: token_offset,
        });
    }

    let program = OsString::from(words.remove(0));
    let args = words.into_iter().map(OsString::from).collect();
    Ok(CommandSpec {
        command: ParsedCommand { program, args },
        stdin,
        stdout,
    })
}

fn redirection_target(
    tokens: &[Token],
    index: usize,
    operator: RedirectionOperator,
    operator_position: usize,
) -> Result<PathBuf, ParseError> {
    match tokens.get(index) {
        Some(Token::Word { value }) => Ok(PathBuf::from(value)),
        Some(Token::Operator { position, .. }) => Err(ParseError {
            kind: ParseErrorKind::MissingRedirectionTarget(operator),
            position: *position,
        }),
        None => Err(ParseError {
            kind: ParseErrorKind::MissingRedirectionTarget(operator),
            position: operator_position,
        }),
    }
}

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

    fn parse_tokens(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        self.skip_whitespace();

        while !self.is_done() {
            tokens.push(self.parse_token()?);
            self.skip_whitespace();
        }

        Ok(tokens)
    }

    fn parse_token(&mut self) -> Result<Token, ParseError> {
        let Some((position, ch)) = self.peek_char() else {
            return Err(ParseError {
                kind: ParseErrorKind::EmptyInput,
                position: self.cursor,
            });
        };

        if let Some(operator) = self.unsupported_operator_at_cursor() {
            return Err(ParseError {
                kind: ParseErrorKind::UnsupportedOperator(operator),
                position,
            });
        }

        if let Some(operator) = self.redirection_operator_at_cursor() {
            self.advance_operator(operator);
            return Ok(Token::Operator {
                kind: operator,
                position,
            });
        }

        if ch.is_whitespace() {
            self.skip_whitespace();
            return self.parse_token();
        }

        let mut output = String::new();
        let mut saw_quoted_part = false;

        while let Some((position, ch)) = self.peek_char() {
            if ch.is_whitespace()
                || self.redirection_operator_at_cursor().is_some()
                || self.unsupported_operator_at_cursor().is_some()
            {
                break;
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

        Ok(Token::Word { value: output })
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
                ';' => Some(UnsupportedOperator::Sequence),
                '`' => Some(UnsupportedOperator::Backtick),
                _ => None,
            }
        }
    }

    fn redirection_operator_at_cursor(&self) -> Option<RedirectionOperator> {
        let rest = &self.input[self.cursor..];
        if rest.starts_with(">>") {
            Some(RedirectionOperator::StdoutAppend)
        } else {
            match rest.chars().next()? {
                '<' => Some(RedirectionOperator::Stdin),
                '>' => Some(RedirectionOperator::StdoutTruncate),
                '|' => Some(RedirectionOperator::Pipe),
                _ => None,
            }
        }
    }

    fn advance_operator(&mut self, operator: RedirectionOperator) {
        let width = match operator {
            RedirectionOperator::StdoutAppend => 2,
            RedirectionOperator::Stdin
            | RedirectionOperator::StdoutTruncate
            | RedirectionOperator::Pipe => 1,
        };
        self.cursor += width;
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
    use std::path::PathBuf;

    use super::{
        CommandSpec, ExecutionPlan, InputSpec, OutputSpec, ParseError, ParseErrorKind,
        ParsedCommand, RedirectionOperator, UnsupportedOperator, parse_execution_line,
    };

    fn parsed(program: &str, args: &[&str]) -> ParsedCommand {
        ParsedCommand {
            program: program.into(),
            args: args.iter().map(Into::into).collect(),
        }
    }

    fn execution_error(input: &str) -> ParseError {
        parse_execution_line(input).expect_err("parse error")
    }

    fn simple_command(input: &str) -> ParsedCommand {
        let ExecutionPlan::Command(command) = parse_execution_line(input).unwrap() else {
            panic!("expected command");
        };

        assert_eq!(command.stdin, InputSpec::Inherit);
        assert_eq!(command.stdout, OutputSpec::Inherit);
        command.command
    }

    fn command_spec(program: &str, args: &[&str]) -> CommandSpec {
        CommandSpec {
            command: parsed(program, args),
            stdin: InputSpec::Inherit,
            stdout: OutputSpec::Inherit,
        }
    }

    #[test]
    fn parses_bare_words() {
        assert_eq!(
            simple_command("cargo --version"),
            parsed("cargo", &["--version"])
        );
    }

    #[test]
    fn parses_quoted_words() {
        assert_eq!(
            simple_command("powershell -NoProfile -Command \"Get-Date\""),
            parsed("powershell", &["-NoProfile", "-Command", "Get-Date"])
        );
    }

    #[test]
    fn preserves_empty_quoted_arguments() {
        assert_eq!(
            simple_command("tool \"\" tail"),
            parsed("tool", &["", "tail"])
        );
    }

    #[test]
    fn preserves_escaped_quotes_inside_quotes() {
        assert_eq!(
            simple_command("tool \"say \\\"hello\\\"\""),
            parsed("tool", &["say \"hello\""])
        );
    }

    #[test]
    fn preserves_escaped_backslashes_inside_quotes() {
        assert_eq!(
            simple_command("tool \"C:\\\\Program Files\""),
            parsed("tool", &["C:\\Program Files"])
        );
    }

    #[test]
    fn keeps_unescaped_backslashes_inside_quotes_literal() {
        assert_eq!(
            simple_command("tool \"C:\\Program Files\""),
            parsed("tool", &["C:\\Program Files"])
        );
    }

    #[test]
    fn ignores_leading_and_trailing_whitespace() {
        assert_eq!(
            simple_command(" \t cargo --version  \r\n"),
            parsed("cargo", &["--version"])
        );
    }

    #[test]
    fn reports_empty_input() {
        assert_eq!(execution_error(" \t ").kind, ParseErrorKind::EmptyInput);
    }

    #[test]
    fn reports_unterminated_quotes() {
        assert_eq!(
            execution_error("tool \"unterminated"),
            ParseError {
                kind: ParseErrorKind::UnterminatedQuote,
                position: 5,
            }
        );
    }

    #[test]
    fn reports_dangling_escapes() {
        assert_eq!(
            execution_error("tool \"dangling\\"),
            ParseError {
                kind: ParseErrorKind::DanglingEscape,
                position: 14,
            }
        );
    }

    #[test]
    fn rejects_unsupported_operators() {
        let cases = [
            ("tool && other", UnsupportedOperator::AndIf, 5),
            ("tool || other", UnsupportedOperator::OrIf, 5),
            ("tool ; other", UnsupportedOperator::Sequence, 5),
            ("tool `date`", UnsupportedOperator::Backtick, 5),
            ("tool $(date)", UnsupportedOperator::CommandSubstitution, 5),
        ];

        for (input, operator, position) in cases {
            assert_eq!(
                execution_error(input),
                ParseError {
                    kind: ParseErrorKind::UnsupportedOperator(operator),
                    position,
                }
            );
        }
    }

    #[test]
    fn parses_stdout_truncate_redirection() {
        let mut expected = command_spec("tool", &["arg"]);
        expected.stdout = OutputSpec::File {
            path: PathBuf::from("out.txt"),
            append: false,
        };

        assert_eq!(
            parse_execution_line("tool arg > out.txt").unwrap(),
            ExecutionPlan::Command(expected)
        );
    }

    #[test]
    fn parses_stdout_append_redirection() {
        let mut expected = command_spec("tool", &[]);
        expected.stdout = OutputSpec::File {
            path: PathBuf::from("out.txt"),
            append: true,
        };

        assert_eq!(
            parse_execution_line("tool >> out.txt").unwrap(),
            ExecutionPlan::Command(expected)
        );
    }

    #[test]
    fn parses_stdin_redirection() {
        let mut expected = command_spec("tool", &[]);
        expected.stdin = InputSpec::File(PathBuf::from("input.txt"));

        assert_eq!(
            parse_execution_line("tool < input.txt").unwrap(),
            ExecutionPlan::Command(expected)
        );
    }

    #[test]
    fn parses_two_command_pipeline() {
        assert_eq!(
            parse_execution_line("producer | consumer arg").unwrap(),
            ExecutionPlan::Pipeline {
                left: command_spec("producer", &[]),
                right: command_spec("consumer", &["arg"]),
            }
        );
    }

    #[test]
    fn reports_missing_redirection_targets() {
        assert_eq!(
            execution_error("tool >").kind,
            ParseErrorKind::MissingRedirectionTarget(RedirectionOperator::StdoutTruncate)
        );
        assert_eq!(
            execution_error("tool < | other").kind,
            ParseErrorKind::MissingRedirectionTarget(RedirectionOperator::Stdin)
        );
    }

    #[test]
    fn reports_multiple_pipelines() {
        assert_eq!(
            execution_error("one | two | three").kind,
            ParseErrorKind::MultiplePipelines
        );
    }
}
