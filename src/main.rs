use std::env;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
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
        String::from_utf8(output.stdout)
            .map_err(|error| format!("rustfmt returned invalid UTF-8: {error}"))
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

fn cache_path(source: &str) -> PathBuf {
    let directory = env::var_os("RUSTFMT_REPL_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("rustfmt-repl-cache"));
    let mut hasher = DefaultHasher::new();
    env!("CARGO_PKG_VERSION").hash(&mut hasher);
    source.hash(&mut hasher);
    directory.join(format!("{:016x}.cache", hasher.finish()))
}

fn read_cache(path: &Path, source: &str) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let length = usize::try_from(u64::from_le_bytes(bytes.get(..8)?.try_into().ok()?)).ok()?;
    let cached_source = bytes.get(8..8 + length)?;
    if cached_source != source.as_bytes() {
        return None;
    }
    String::from_utf8(bytes.get(8 + length..)?.to_vec()).ok()
}

fn write_cache(path: &Path, source: &str, formatted: &str) {
    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    let mut bytes = Vec::with_capacity(8 + source.len() + formatted.len());
    bytes.extend_from_slice(&(source.len() as u64).to_le_bytes());
    bytes.extend_from_slice(source.as_bytes());
    bytes.extend_from_slice(formatted.as_bytes());
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    if fs::write(&temporary, bytes).is_ok() {
        if fs::rename(&temporary, path).is_err() {
            let _ = fs::remove_file(temporary);
        }
    }
}

fn format_source_cached(source: &str) -> Result<String, String> {
    let path = cache_path(source);
    if let Some(formatted) = read_cache(&path, source) {
        return Ok(formatted);
    }
    let formatted = format_source(source)?;
    write_cache(&path, source, &formatted);
    Ok(formatted)
}

fn main() {
    let mut cache = false;
    let mut path = None;
    for argument in env::args_os().skip(1) {
        if argument == "--cache" {
            cache = true;
        } else if path.is_none() {
            path = Some(PathBuf::from(argument));
        } else {
            eprintln!("usage: rustfmt-repl [--cache] [path]");
            std::process::exit(2);
        }
    }
    let source = match &path {
        Some(path) => fs::read_to_string(path).unwrap_or_else(|error| {
            eprintln!("failed to read {}: {error}", path.display());
            std::process::exit(2);
        }),
        None => {
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .unwrap_or_else(|error| {
                    eprintln!("failed to read stdin: {error}");
                    std::process::exit(2);
                });
            source
        }
    };

    let formatted = if cache {
        format_source_cached(&source)
    } else {
        format_source(&source)
    }
    .unwrap_or_else(|error| {
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
