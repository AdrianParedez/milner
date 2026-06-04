use std::io::{self, BufRead, Write};

use super::parser::parse_command_line;
use super::{RunError, execute_command};

const PROMPT: &str = "keel> ";

pub fn run_prompt() -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let stderr = io::stderr();

    match run_prompt_with_io(stdin.lock(), stdout.lock(), stderr.lock()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("run: {err}");
            125
        }
    }
}

struct PromptState {
    last_status: i32,
}

fn run_prompt_with_io<R, W, E>(mut input: R, mut output: W, mut errors: E) -> Result<i32, io::Error>
where
    R: BufRead,
    W: Write,
    E: Write,
{
    let mut state = PromptState { last_status: 0 };
    let mut line = String::new();

    loop {
        output.write_all(PROMPT.as_bytes())?;
        output.flush()?;

        line.clear();
        let bytes = input.read_line(&mut line)?;
        if bytes == 0 {
            return Ok(state.last_status);
        }

        let command_line = trim_line_ending(&line);
        if command_line.trim().is_empty() {
            continue;
        }

        match parse_command_line(command_line) {
            Ok(command) => match execute_command(command) {
                Ok(code) => state.last_status = code as i32,
                Err(err) => {
                    writeln!(errors, "run: {err}")?;
                    state.last_status = err.exit_code();
                }
            },
            Err(err) => {
                writeln!(errors, "run: {err}")?;
                state.last_status = RunError::Parse(err).exit_code();
            }
        }
    }
}

fn trim_line_ending(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::run_prompt_with_io;

    #[test]
    fn empty_input_prompts_again_until_eof() {
        let mut output = Vec::new();
        let mut errors = Vec::new();

        let code = run_prompt_with_io(Cursor::new("\n"), &mut output, &mut errors).unwrap();

        assert_eq!(code, 0);
        assert_eq!(String::from_utf8(output).unwrap(), "keel> keel> ");
        assert_eq!(String::from_utf8(errors).unwrap(), "");
    }

    #[test]
    fn parser_errors_are_non_fatal() {
        let mut output = Vec::new();
        let mut errors = Vec::new();

        let code = run_prompt_with_io(Cursor::new("tool |\n"), &mut output, &mut errors).unwrap();

        assert_eq!(code, 2);
        assert_eq!(String::from_utf8(output).unwrap(), "keel> keel> ");
        assert!(
            String::from_utf8(errors)
                .unwrap()
                .contains("unsupported operator `|`")
        );
    }
}
