# toolboxes

Two ad-hoc scripting toolboxes for AI coding agents (Claude Code and others)
on Windows with WSL2. Rust is the default; Python covers tasks where its
libraries are clearly better.

| File | Purpose |
|---|---|
| `SETUP.md` | Step-by-step setup for an agent: install, precompile, verify |
| `INSTRUCTIONS.md` | Rules to add to the agent's global instruction file |
| `Cargo.toml`, `Cargo.lock` | Rust toolbox: about 70 precompiled crates |
| `rs` | Wrapper that runs single-file Rust scripts against the toolbox |
| `.cargo/config.toml` | mold linker for faster builds |
| `python/requirements.txt` | Package list of the Python tool venv on Windows |
| `backup.sh` | Copies the Python list here, commits and pushes |

To set up a machine, give an agent this repository and ask it to follow
`SETUP.md`.

The repository is checked out as `~/.rust-tools` in WSL. Run
`./backup.sh "<conventional commit title>"` after changing either toolbox.
