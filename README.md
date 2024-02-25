
# astronote

astronote is a CLI spaced-repetition software to review files.

You can
- run it in your shell.
- review (and edit) your plain-text files with your favorite text editor (such as NeoVim).
- determine next datetime to review each file with spaced-repetition algorithm SuperMemo2.

## Installation

With [`cargo-make`](https://github.com/sagiegurari/cargo-make):
```sh
# In `astronote` directory
cargo make install
```

Without `cargo-make`:
```sh
# In `astronote` directory
cargo install --path ./astronote-cli
```

## Usage

Add files.
```sh
astronote add -f /path/to/file
```

Review them.
```sh
astronote review -n <num>
```

You can create configuration file (`.astronote.toml`).
```toml
database_path = "/path/to/your/database/file"
editor_command = "your_favorite_editor"
```
