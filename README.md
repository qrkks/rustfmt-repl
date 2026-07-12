# rustfmt-snippet

A tiny adapter that lets `rustfmt` format Rust statement snippets used in
Markdown code blocks and REPLs.

It first runs `rustfmt` on the input as a complete Rust source file. If that
fails, it temporarily wraps the input in `fn main()`, formats it, and removes
the wrapper again.

## Build

```powershell
cargo build --release
Copy-Item .\target\release\rustfmt-snippet.exe $HOME\.cargo\bin\
```

## Usage

Read from stdin and write to stdout:

```powershell
'let x=(1,2); dbg!(x,    x.0);' | rustfmt-snippet
```

Or format a file in place:

```powershell
rustfmt-snippet path\to\snippet.rs
```

## mdsf

```json
{
  "languages": {
    "rust": {
      "binary": "C:\\Users\\dell\\.cargo\\bin\\rustfmt-snippet.exe",
      "arguments": ["$PATH"],
      "stdin": false
    }
  }
}
```
