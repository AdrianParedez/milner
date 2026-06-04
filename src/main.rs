#[cfg(windows)]
mod process;

#[cfg(windows)]
fn main() {
    match process::run_from_env() {
        Ok(code) => std::process::exit(code as i32),
        Err(err) => {
            eprintln!("run: {err}");
            std::process::exit(err.exit_code());
        }
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("run: this milestone is Windows-only");
    std::process::exit(125);
}
