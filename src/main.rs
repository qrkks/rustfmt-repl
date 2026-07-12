use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const START: &str = "// __RUSTFMT_SNIPPET_START__";
const END: &str = "// __RUSTFMT_SNIPPET_END__";

fn run_rustfmt(source: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .args(["--emit", "stdout", "--edition", "2024"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start rustfmt: {error}"))?;

    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(source.as_bytes())
        .map_err(|error| format!("failed to write to rustfmt: {error}"))?;

    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for rustfmt: {error}"))?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|error| format!("rustfmt returned invalid UTF-8: {error}"))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

fn wrap(source: &str) -> String {
    let mut wrapped = String::from("fn main() {\n");
    wrapped.push_str(START);
    wrapped.push('\n');
    wrapped.push_str(source);
    if !source.ends_with('\n') {
        wrapped.push('\n');
    }
    wrapped.push_str(END);
    wrapped.push_str("\n}\n");
    wrapped
}

fn unwrap(formatted: &str, original_had_newline: bool) -> Result<String, String> {
    let lines: Vec<&str> = formatted.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.trim() == START)
        .ok_or_else(|| "rustfmt output lost the start marker".to_owned())?;
    let end = lines
        .iter()
        .position(|line| line.trim() == END)
        .ok_or_else(|| "rustfmt output lost the end marker".to_owned())?;

    if end <= start {
        return Err("rustfmt output contains invalid marker order".to_owned());
    }

    let body = &lines[start + 1..end];
    let mut result = body
        .iter()
        .map(|line| line.strip_prefix("    ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");

    if original_had_newline {
        result.push('\n');
    }
    Ok(result)
}

fn format_source(source: &str) -> Result<String, String> {
    if let Ok(formatted) = run_rustfmt(source) {
        return Ok(formatted);
    }

    let wrapped = wrap(source);
    let formatted = run_rustfmt(&wrapped)?;
    unwrap(&formatted, source.ends_with('\n'))
}

fn main() {
    let path = env::args_os().nth(1).map(PathBuf::from);
    let source = match &path {
        Some(path) => fs::read_to_string(path).unwrap_or_else(|error| {
            eprintln!("failed to read {}: {error}", path.display());
            std::process::exit(2);
        }),
        None => {
            let mut source = String::new();
            io::stdin().read_to_string(&mut source).unwrap_or_else(|error| {
                eprintln!("failed to read stdin: {error}");
                std::process::exit(2);
            });
            source
        }
    };

    let formatted = format_source(&source).unwrap_or_else(|error| {
        eprintln!("unable to format Rust source or snippet:\n{error}");
        std::process::exit(1);
    });

    match path {
        Some(path) => fs::write(&path, formatted).unwrap_or_else(|error| {
            eprintln!("failed to write {}: {error}", path.display());
            std::process::exit(2);
        }),
        None => print!("{formatted}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwraps_a_formatted_statement_snippet() {
        let formatted = format_source("let x = (1,2);\ndbg!(x,       x.0);\n").unwrap();
        assert_eq!(formatted, "let x = (1, 2);\ndbg!(x, x.0);\n");
    }

    #[test]
    fn keeps_complete_programs_complete() {
        let formatted = format_source("fn main(){let x=(1,2);}\n").unwrap();
        assert_eq!(formatted, "fn main() {\n    let x = (1, 2);\n}\n");
    }
}
