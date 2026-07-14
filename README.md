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

## mdsf

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

Make sure Cargo's binary directory is available on `PATH`. Its usual location
is `$HOME/.cargo/bin` on Unix-like systems and `%USERPROFILE%\.cargo\bin` on
Windows. If the editor does not inherit your shell environment, use the full
path to the executable in your local configuration.
