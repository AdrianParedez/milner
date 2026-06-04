use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;

use super::RunError;

pub fn build_command_line(program: &OsStr, args: &[OsString]) -> Result<Vec<u16>, RunError> {
    let mut command = Vec::new();
    append_quoted_arg(&mut command, program)?;

    for arg in args {
        command.push(' ' as u16);
        append_quoted_arg(&mut command, arg.as_os_str())?;
    }

    command.push(0);
    Ok(command)
}

fn append_quoted_arg(output: &mut Vec<u16>, arg: &OsStr) -> Result<(), RunError> {
    let encoded: Vec<u16> = arg.encode_wide().collect();
    if encoded.contains(&0) {
        return Err(RunError::InteriorNul);
    }

    let needs_quotes = encoded.is_empty()
        || encoded
            .iter()
            .any(|unit| *unit == ' ' as u16 || *unit == '\t' as u16 || *unit == '"' as u16);

    if !needs_quotes {
        output.extend(encoded);
        return Ok(());
    }

    output.push('"' as u16);
    let mut backslashes = 0usize;

    for unit in encoded {
        if unit == '\\' as u16 {
            backslashes += 1;
            continue;
        }

        if unit == '"' as u16 {
            append_repeated(output, '\\' as u16, backslashes * 2 + 1);
            output.push(unit);
            backslashes = 0;
            continue;
        }

        append_repeated(output, '\\' as u16, backslashes);
        backslashes = 0;
        output.push(unit);
    }

    append_repeated(output, '\\' as u16, backslashes * 2);
    output.push('"' as u16);
    Ok(())
}

fn append_repeated(output: &mut Vec<u16>, unit: u16, count: usize) {
    output.extend(std::iter::repeat_n(unit, count));
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};

    use super::build_command_line;

    fn command_line(program: &str, args: &[&str]) -> String {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();
        let command = build_command_line(OsStr::new(program), &args).expect("command line");
        String::from_utf16(&command[..command.len() - 1]).expect("utf16")
    }

    #[test]
    fn leaves_simple_arguments_unquoted() {
        assert_eq!(command_line("cargo", &["--version"]), "cargo --version");
    }

    #[test]
    fn quotes_arguments_with_spaces() {
        assert_eq!(
            command_line("powershell", &["-Command", "Get-Date"]),
            "powershell -Command Get-Date"
        );
        assert_eq!(
            command_line("tool", &["hello world"]),
            "tool \"hello world\""
        );
    }

    #[test]
    fn quotes_empty_arguments() {
        assert_eq!(command_line("tool", &[""]), "tool \"\"");
    }

    #[test]
    fn doubles_trailing_backslashes_inside_quotes() {
        assert_eq!(
            command_line("tool", &["C:\\path with space\\"]),
            "tool \"C:\\path with space\\\\\""
        );
    }

    #[test]
    fn escapes_quotes_and_preceding_backslashes() {
        assert_eq!(command_line("tool", &["say\"hi"]), "tool \"say\\\"hi\"");
        assert_eq!(
            command_line("tool", &["slashes\\\\\"quote"]),
            "tool \"slashes\\\\\\\\\\\"quote\""
        );
    }
}
