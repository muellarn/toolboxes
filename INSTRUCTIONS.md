# Agent instructions: ad-hoc scripting with Rust and Python

Copy everything below the line into the agent's global instruction file
(for Claude Code: `%USERPROFILE%\.claude\CLAUDE.md`; other agents: their
equivalent, such as `AGENTS.md`). Replace the placeholders with this
machine's values first:

| Placeholder | Meaning | Example |
|---|---|---|
| `<WIN_HOME>` | Windows user profile (`%USERPROFILE%`) | `C:\Users\<name>` |
| `<WIN_HOME_WSL>` | the same folder seen from WSL | `/mnt/c/Users/<name>` |
| `<WSL_HOME>` | home directory in WSL (`$HOME`) | `/home/<user>` |
| `<WSL_HOME_UNC>` | WSL home seen from Windows | `\\wsl.localhost\<distro>\home\<user>` |
| `<REPO>` | GitHub repository of this toolbox | `<owner>/<repo>` |

---

## Ad-hoc scripting: Rust first, Python only where its libraries are better

- This applies on Windows and in WSL, whatever shell runs the command. Ad-hoc
  work (file operations beyond the dedicated tools, data processing,
  Office/PDF/JSON handling, HTTP requests, system inspection) uses one of the
  two toolboxes below instead of PowerShell, the system `python3` in WSL,
  `node -e`, or `perl`. Use PowerShell only when neither toolbox fits. Calling
  the toolboxes or native executables such as `git`, `gh`, or `cargo` directly
  is not affected.
- "Python" in this section means the tool venv below, never the system
  `python3` in WSL or a project's own `.venv/`.
- Write script files to the session scratchpad when the harness provides one,
  otherwise to `%TEMP%`, and delete them when they are no longer needed. Their
  data follows "File access across WSL and Windows": WSL scripts keep working
  files under `/tmp` and touch the Windows side only for inputs and final
  results.
- The toolboxes are for tooling only. Real projects keep their own `.venv/` or
  `Cargo.toml`; never add project dependencies to a toolbox.
- Both toolboxes are backed up in the repo `<REPO>`, checked out as
  `~/.rust-tools` (the Python package list is copied to
  `python/requirements.txt`). It also holds `INSTRUCTIONS.md` (this file) and
  `rules/` (the only copy of the shared rules, see "Shared rules" at the end).
  After changing a toolbox, these instructions, or `rules/`:
  1. Mirror changes of these instructions in `INSTRUCTIONS.md`; change shared
     rules only in `rules/`.
  2. If the repo is public, make sure no tracked file contains personal names,
     user paths, or secrets (`git grep -niE '<name>|/Users/[a-z]|/home/[a-z]'`).
  3. Run `~/.rust-tools/backup.sh "<conventional commit title>"` in WSL. It
     stages everything, commits, and pushes directly to `main`.

### Choosing Rust or Python

- Rust is the default, including one-off scripts, quick inspections, one-liners
  (`rs -e`), and single files on the Windows side (read through `/mnt/c/...`
  with buffered or whole-file I/O). It compiles in 1–2.5 s, runs faster, and
  the compiler catches errors before a long run.
- Rust or `tb` also replaces shell pipelines and loops (`grep | sed | awk`,
  `sort | uniq -c`, `find -exec`, `xargs`, `for`/`while`, `jq`, `cut`, `wc`) as
  soon as the work involves a loop, a pipe that sorts, counts, or filters, any
  evaluation of the output, or may take longer than about a second. Simple
  single commands stay as they are (`ls`, `cat`/`head` of a small file,
  `mkdir`, `cp`, `mv`, `rm`, `git`, `df`, `du`, one `grep` or `find`), as do the
  agent's dedicated search, read, and edit tools; several independent ones may
  share one call. When unsure, use `rs -e`.
- For common cases use the precompiled `tb` first; it starts instantly:
  `tb json` (pretty-print, `-p /json/pointer`, `-k` keys or length; also JSON5,
  YAML, TOML), `tb conv FILE --to json|yaml|toml|csv`, `tb csv FILE` (columns,
  types, empty cells, min/max/mean, `-n` first rows), `tb xlsx FILE` (sheet as
  CSV, `-l` lists sheets), `tb count` (replaces `sort | uniq -c | sort -rn`,
  `-r` counts the first regex group), and `tb size DIR` (size and file count
  per entry). `tb <command> --help` shows all options. Text search stays with
  `rg`.
- Structured data (JSON, CSV, XML, TOML, YAML) is always read with a parser in
  `tb` or `rs`, never with `grep`/`sed`/`awk` or other regex matching, even for
  a quick look. Only `head`/`wc` to peek at raw bytes or line counts are fine.
- Use Python only where its libraries are clearly better: Word and PowerPoint,
  PDF of any kind (reading, creating, editing, forms), editing existing Excel
  files, mail files (`.msg`, `.eml`), OCR, plots, machine learning and
  statistics, LLM SDKs, and Windows-specific work (COM, WMI, clipboard, GUI
  automation), which Rust cannot do because it runs in WSL. Reading Excel files
  and writing new ones stays with Rust (calamine reads, rust_xlsxwriter only
  creates new files). Decide by library quality, not by habit or script length.
- Walk or search large directory trees on the Windows side (thousands of files)
  with Windows-native tools (`rg`, Git Bash `grep`/`find`, or the Python venv),
  not with Rust in WSL, because every file access across the boundary is slow.
- Use one language per task: when part of a small task needs Python, do all of
  it in Python. Split only real data pipelines, handing data between a Python
  and a Rust step as Parquet or CSV files.

### Rust toolbox (WSL)

- `~/.rust-tools` in WSL (from Windows: `<WSL_HOME_UNC>\.rust-tools`) is a
  Cargo project with precompiled dependencies. The wrapper `rs`
  (`~/.local/bin/rs`) copies a single-file script into it, builds it with the
  `script` profile (release, incremental, mold linker), and runs it in the
  caller's working directory.
- Run in WSL: `rs script.rs [args]`, or `rs -e 'code'` for a snippet that becomes
  the body of `fn main() -> anyhow::Result<()>` (`Context`, `Result`, `bail` are
  imported). The wrapper appends `Ok(())` itself; ending the snippet with
  `Ok(())` fails with error E0282.
- From Windows: `wsl.exe -e <WSL_HOME>/.local/bin/rs <script> [args]`, and the
  same for `tb`. In Git Bash prefix `MSYS_NO_PATHCONV=1`, otherwise `/home/...`
  and `/mnt/...` arguments are rewritten to Windows paths. A Windows script path
  (`C:\...`) is converted, other arguments are passed unchanged, so give the
  script `/mnt/c/...` paths. `wsl.exe` refuses to start from a very long Windows
  working directory; `cd` to a short one first.
- Start time (measured): unchanged script 20–60 ms in WSL (about 0.2 s via
  `wsl.exe`); new or edited script about 1 s, with polars, calamine,
  rust_xlsxwriter, or zip about 2.5–3.5 s. Image scripts take about 1 s through
  `tb::img`, but about 14 s with `image::open` or `.save()`. Concurrent `rs`
  builds wait for each other (one cargo lock). Only the precompile after
  changing `Cargo.toml` is slow (polars alone takes several minutes), so never
  interrupt a running `rs` build or `rs --warmup`.
- A script is a plain `fn main()` file with `use` statements for toolbox crates
  and no `[dependencies]` header. Prefer `anyhow::Result` and `?`. Compiler
  messages point to the script's own path; unused-item warnings are off.
- Load and save images with the precompiled wrappers in the toolbox library
  `tb` (`~/.rust-tools/src/lib.rs`): `tb::img::open(path)`,
  `tb::img::load(&bytes)`, `tb::img::save(&dynamic_image, path)`,
  `tb::img::save_as(.., ImageFormat)`, `tb::img::encode(&img, ImageFormat) ->
  Vec<u8>`, and `tb::img::save_rgb`, `save_rgba`, `save_gray` for
  `RgbImage`/`RgbaImage`/`GrayImage`. Everything else (resize, crop, pixels,
  imageproc) uses `image`/`imageproc` directly.
- Do not write a crate API from memory when unsure of it. Look up the exact
  version in `~/.rust-tools/Cargo.lock`, then read the examples, README, and
  signatures in `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/`.
- Usable directly (full list with versions in `~/.rust-tools/Cargo.toml`):
  - Errors/CLI/output: anyhow, clap (derive), indicatif, comfy-table,
    owo-colors, humantime, bytesize
  - Formats: serde (derive), serde_json, serde_yaml_ng, toml, json5, quick-xml
    (serialize), roxmltree (easy XML reading), csv, dotenvy
  - Data/math: polars (lazy, csv, parquet, json, sql, strings, temporal),
    ndarray, statrs, rand, itertools, petgraph, rayon, geo (geometry and
    distances), contour (contour lines and isobands from grids), plotters
    (simple PNG/SVG charts)
  - Excel/PDF: calamine (read xlsx/xls/xlsb/ods), rust_xlsxwriter, lopdf,
    pdf-extract
  - Web/HTML/Markdown: reqwest (blocking, json), tokio (full), scraper, url,
    html2text, pulldown-cmark, minijinja
  - Images: image, imageproc (draw shapes and text, filters), ab_glyph (fonts),
    kamadak-exif, qrcode
  - Text/dates: regex, fancy-regex, strsim, similar (diffs),
    unicode-normalization, deunicode, slug, encoding_rs, chardetng, textwrap,
    chrono (serde), chrono-tz
  - Files/system: walkdir, ignore, glob, tempfile, memmap2, notify, infer,
    sysinfo, zip, flate2, zstd, tar
  - Hashing/ids: sha2, md-5, blake3, hex, base64, uuid (v4)
  - Databases: rusqlite (bundled SQLite)
- Extend: `cd ~/.rust-tools && cargo add <crate> [--features ...]`, move the new
  line into the matching section of `Cargo.toml`, then run `rs --warmup` once.
  Prefer pure-Rust crates with minimal features; changing `Cargo.toml` makes
  each script rebuild once. Add a wrapper to `src/lib.rs` only for a call whose
  first build was measured above about 3 s; changing `src/lib.rs` relinks every
  script once. Changes to `src/bin/tb.rs` take effect after `rs --warmup`.
- `rs --clean` deletes cached script copies and binaries when `target/` grows.

### Python tool venv (Windows)

- Interpreter: `<WIN_HOME>\.venv-tools\Scripts\python.exe`
  (from WSL: `<WIN_HOME_WSL>/.venv-tools/Scripts/python.exe`).
  Run short snippets inline with `python.exe -c "..."`.
- Usable directly (full list in `<WIN_HOME>\.venv-tools\requirements.txt`):
  - Excel/Office: openpyxl, xlsxwriter, xlrd, pyxlsb, xlwings, python-docx,
    python-pptx, odfpy, mammoth, extract-msg, markitdown, pypandoc_binary
  - PDF/OCR: pypdf, pdfplumber, pymupdf, pikepdf, pypdfium2, reportlab, fpdf2,
    tabula-py, rapidocr-onnxruntime
  - Data/math/plots: numpy, pandas, polars, pyarrow, duckdb, scipy,
    scikit-learn, statsmodels, sympy, networkx, pint, shapely, matplotlib,
    seaborn, plotly, tabulate, csvkit
  - Formats: orjson, json5, jsonschema, jmespath, deepdiff, pydantic, pyyaml,
    ruamel.yaml, tomlkit, xmltodict, lxml, python-dotenv
  - Web/HTML/Markdown: requests, httpx, aiohttp, websockets, beautifulsoup4,
    selectolax, trafilatura, feedparser, markdown, markdownify, html2text,
    jinja2, yt-dlp
  - Media: pillow, pillow-heif, opencv-python-headless, imageio, exifread,
    qrcode, python-barcode, pydub, mutagen, ffmpeg-python
  - Text/dates: regex, rapidfuzz, chardet, ftfy, unidecode, python-slugify,
    python-dateutil, babel, num2words, phonenumbers, icalendar, pygments
  - Files/system/Windows: psutil, send2trash, watchdog, pathspec, filetype,
    py7zr, rarfile, zstandard, pyperclip, pywin32, comtypes, wmi, pywinauto
  - Databases: sqlalchemy, sqlite-utils, psycopg, pymysql, pyodbc
  - Network/crypto/git: cryptography, paramiko, pyjwt, gitpython
  - CLI/dev/LLM: rich, tqdm, typer, ipython, pytest, ruff, mypy, anthropic,
    openai, tiktoken
- Extend: `python.exe -m pip install <package>`, then add it to the matching
  section of `requirements.txt`.

## Shared rules

The files in `~/.rust-tools/rules/` belong to these instructions and are the
only copy of them: `running-commands.md` (efficient scripts, runtime estimates,
background runs, progress reports) and `wsl-windows-files.md` (file access
across WSL and Windows). Load them instead of copying them, so a change there
reaches every agent:

- Claude Code: import them at the end of the global `CLAUDE.md`, one per line,
  as `@//wsl.localhost/<distro>/home/<user>/.rust-tools/rules/running-commands.md`
  (forward slashes only: Claude Code silently ignores an `@` path containing
  a backslash, and of all network paths it follows only `//wsl.localhost/`
  and `//wsl$/`).
- OpenCode: list them in the `instructions` array of the global
  `opencode.json`, for example
  `"\\\\wsl.localhost\\<distro>\\home\\<user>\\.rust-tools\\rules\\running-commands.md"`
  on Windows or `"/home/<user>/.rust-tools/rules/running-commands.md"` in
  Linux.
- Other agents: copy the files' contents into their instruction file.
