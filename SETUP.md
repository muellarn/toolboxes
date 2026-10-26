# Setup guide for an AI agent

You are setting up two ad-hoc scripting toolboxes on a Windows machine with
WSL2, so that you (and later sessions) can use them right away:

- **Rust toolbox** in WSL: a Cargo project with about 75 precompiled crates and
  the wrapper `rs`. This is the default for all scripting.
- **Python tool venv** on Windows: for tasks where Python's libraries are
  clearly better (Office, OCR, plots, ML, LLM SDKs, Windows automation).

Placeholders such as `<REPO>`, `<WSL_HOME>` and `<WSL_HOME_UNC>` are defined
in the table at the top of `INSTRUCTIONS.md`.

Work through the steps in order. Ask the user before installing system
packages, before running anything with `sudo`, and before changing the agent's
global instruction file. If the toolboxes already exist, update them instead of
reinstalling (`git pull`, then step 4 and step 6).

## 1. Check prerequisites

- Windows 10/11 with WSL2 and an Ubuntu distribution: `wsl.exe -l -v`.
- In WSL, the build tools (ask the user to approve the `sudo` command):

      sudo apt-get update
      sudo apt-get install -y build-essential clang mold cmake pkg-config git libfontconfig1-dev

- Rust via rustup in WSL (skip if `cargo --version` works):

      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

- Only for backups with `backup.sh`: a GitHub repository the user can push to
  (their own fork or copy of this one) and GitHub access in WSL, for example
  with `gh auth login` (GitHub CLI).
- On Windows, Python 3 with the `py` launcher (`py --version`). Use the newest
  Python version for which all packages in `python/requirements.txt` install.

## 2. Clone the repository in WSL

    git clone https://github.com/<REPO>.git ~/.rust-tools

`<REPO>` is the user's own fork or copy if they want backups, otherwise the
public original. For backups, also let git use the GitHub CLI credentials:

    git -C ~/.rust-tools config credential.helper '!gh auth git-credential'

The toolbox must live on the WSL filesystem (`~`), not under `/mnt/c`, or
builds become very slow.

## 3. Put `rs` and `tb` on the PATH

    mkdir -p ~/.local/bin
    ln -sfn ~/.rust-tools/rs ~/.local/bin/rs
    ln -sfn ~/.rust-tools/target/script/tb ~/.local/bin/tb
    chmod +x ~/.rust-tools/rs ~/.rust-tools/backup.sh

The `tb` link points to the binary that step 4 builds.

Ubuntu adds `~/.local/bin` to the PATH of login shells once the directory
exists. Calls from Windows use the absolute path, so they do not depend on it.

## 4. Precompile all crates

    ~/.local/bin/rs --warmup

This takes about 10–15 minutes on first run (polars alone takes several
minutes) and needs about 2 GB of disk space. Run it in the background and wait
for it to finish. **Never interrupt it**: an interrupted build leaves partial
artifacts that have to be rebuilt.

## 5. Verify the Rust toolbox

In WSL, run this twice; the second run must take well under 0.1 s:

    time ~/.local/bin/rs -e 'println!("{}", serde_json::json!({"ok": true}))'

Then check the precompiled tool and library:

    echo '{"ok": true}' | ~/.local/bin/tb json -p /ok
    ~/.local/bin/rs -e 'println!("{}", tb::img::encode(&image::DynamicImage::new_rgb8(2, 2), image::ImageFormat::Png)?.len())'

From Windows (Git Bash; PowerShell needs no `MSYS_NO_PATHCONV`):

    MSYS_NO_PATHCONV=1 wsl.exe -e <WSL_HOME>/.local/bin/rs -e 'println!("ok")'

If the second run is slow or recompiles crates, something in the environment
differs between runs; `rs` already starts cargo with a fixed environment, so
check that `~/.rust-tools/.cargo/config.toml` and `Cargo.lock` are unchanged.

## 6. Create the Python tool venv on Windows

In cmd (in PowerShell write `$env:USERPROFILE` instead of `%USERPROFILE%`);
the requirements file is read from the WSL checkout:

    py -3 -m venv %USERPROFILE%\.venv-tools
    %USERPROFILE%\.venv-tools\Scripts\python.exe -m pip install --upgrade pip
    copy <WSL_HOME_UNC>\.rust-tools\python\requirements.txt %USERPROFILE%\.venv-tools\requirements.txt
    %USERPROFILE%\.venv-tools\Scripts\python.exe -m pip install -r %USERPROFILE%\.venv-tools\requirements.txt

The copy in `%USERPROFILE%\.venv-tools\requirements.txt` is the file to edit
later; `backup.sh` copies it back into the repository. If single packages fail
to install (for example because no wheel exists yet for a very new Python),
install the rest, then report the failed ones to the user instead of silently
dropping them.

Verify:

    %USERPROFILE%\.venv-tools\Scripts\python.exe -c "import pandas, openpyxl, docx, pptx, fitz, matplotlib; print('ok')"

## 7. Install the agent instructions

`INSTRUCTIONS.md` contains the rules that make an agent use these toolboxes
(Rust as default, when to use Python, how to call `rs` from Windows, how to
extend and back up, and the rules for file access across WSL and Windows).

1. Fill in the placeholders listed at the top of `INSTRUCTIONS.md` with this
   machine's values.
2. Append everything below its horizontal line to the agent's global
   (user-level, not per-repository) instruction file. If that file already
   contains sections with the same headings, replace those sections and keep
   everything else. Known locations:
   - Claude Code: `%USERPROFILE%\.claude\CLAUDE.md`.
   - GitHub Copilot in VS Code: a user-level instructions file such as
     `toolboxes.instructions.md` with front matter `applyTo: "**"` in the
     VS Code user profile's `prompts` folder (create it via the command
     "Chat: New Instructions File", scope "User").
   - GitHub Copilot CLI: `~/.copilot/copilot-instructions.md`.
   - Other agents: their documented global instruction file (often
     `AGENTS.md`). Check the agent's current documentation when unsure;
     these locations change between versions.
3. Show the user the inserted text.

Instruction files are usually read only when a session starts, so new rules
take effect in the next session.

## 8. Report

Tell the user:

- whether the Rust warmup finished and the measured start times from step 5,
- which Python packages, if any, failed to install,
- where the instructions were added,
- that `~/.rust-tools/backup.sh "<conventional commit title>"` backs up both
  toolboxes after changes.
