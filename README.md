# rustfmt-repl

A tiny adapter that lets `rustfmt` format Rust statement snippets used in
Markdown code blocks and REPLs.

It first runs `rustfmt` on the input as a complete Rust source file. If that
fails, it temporarily wraps the input in `fn main()`, formats it, and removes
the wrapper again.

## Build

```powershell
cargo install rustfmt-repl
```

## Usage

Read from stdin and write to stdout:

```powershell
'let x=(1,2); dbg!(x,    x.0);' | rustfmt-repl
```

Or format a file in place:

```powershell
rustfmt-repl path\to\snippet.rs
```

For repeated formatting of many snippets, enable the opt-in content cache:

```powershell
rustfmt-repl --cache path\to\snippet.rs
```

## Markdown code-block formatters

`rustfmt-repl` can be used as the Rust formatter behind either
[mdsf](https://github.com/hougesen/mdsf) or
[Panache](https://github.com/jolars/panache). The orchestrator extracts Rust
code blocks; `rustfmt-repl` makes complete files and statement snippets usable
with `rustfmt`.

Use one of these integrations in a formatting chain, not both.

### mdsf

Add the following language entry to `mdsf.json`:

```json
{
  "languages": {
    "rust": {
      "binary": "rustfmt-repl",
      "arguments": ["--cache", "$PATH"],
      "stdin": false
    }
  }
}
```

mdsf gives `rustfmt-repl` a temporary file through `$PATH`, so this example
uses file mode.

### Panache

Add the following entries to `panache.toml`:

```toml
external-max-parallel = 8

[formatters]
rust = "rustfmt-repl"
rs = "rustfmt-repl"

[formatters.rustfmt-repl]
cmd = "rustfmt-repl"
args = ["--cache"]
stdin = true
```

Panache sends each code block through stdin and can run independent code-block
formatters in parallel. `rustfmt-repl` still performs the snippet wrapping and
unwrapping required by `rustfmt`.

## PATH

Make sure Cargo's binary directory is available on `PATH`. Its usual location
is `$HOME/.cargo/bin` on Unix-like systems and `%USERPROFILE%\.cargo\bin` on
Windows. If the editor does not inherit your shell environment, use the full
path to the executable in your local configuration.
