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

- This applies on Windows and in WSL alike, whatever shell runs the command.
  Do ad-hoc work (file and directory operations beyond the dedicated tools,
  data processing, Excel/PDF/Office/JSON handling, HTTP requests, system
  inspection) with one of the two preinstalled toolboxes below. They replace
  PowerShell cmdlets, pipelines, or scripts as well as one-liners in other
  interpreters, such as the system `python3` in WSL, `node -e`, or `perl`.
  Use PowerShell only when neither toolbox fits. Invoking the toolboxes and
  native executables such as `git`, `gh`, or `cargo` directly from the shell
  is not affected by this rule.
- "Python" in this section always means the tool venv below, never the system
  `python3` in WSL or a project's own `.venv/`.
- Write ad-hoc scripts to the temp directory (the session scratchpad when the
  harness provides one, otherwise `%TEMP%`), run them from there, and delete
  them as soon as they are no longer needed.
- Both toolboxes are for tooling only. Real projects keep their own `.venv/` or
  `Cargo.toml`; never add project dependencies to a toolbox.
- Both are backed up in the repo `<REPO>` (checked out as
  `~/.rust-tools`; the Python list is copied to `python/requirements.txt`).
  After changing either toolbox, run `~/.rust-tools/backup.sh "<conventional
  commit title>"` in WSL. It commits directly to `main`; this backup-only repo
  needs no issue or pull request. When these instructions change, update
  `INSTRUCTIONS.md` in the repo as well; shared rules change only in
  `rules/` (see "Shared rules" at the end).

### Choosing Rust or Python

- Rust is the default for everything, including one-off scripts, quick
  inspections and one-liners (`rs -e`), and for individual files on the
  Windows side (read them through `/mnt/c/...` with buffered or whole-file
  I/O). It compiles in 1–2.5 s, runs faster, and the compiler catches errors
  before a long run.
- Rust also replaces Linux shell pipelines and loops (`grep | sed | awk`,
  `sort | uniq -c`, `find -exec`, `xargs`, `for`/`while` loops, `jq`, `cut`,
  `wc` chains) whenever the work is more than one simple command or may take
  longer than about a second: many or large files, counting and aggregating,
  transforming text or structured data, repeated operations. Simple single
  commands stay as they are: `ls`, `cat`/`head` of a small file, `mkdir`,
  `cp`, `mv`, `rm`, `git`, a single `grep` or `find`, and the agent's
  dedicated search, read and edit tools. When unsure, write `rs -e`.
  Several independent read-only checks, each a single simple command (for
  example `ls`, `df`, one `find` or `du`), may share one call. As soon as a
  loop, a pipe that sorts, counts or filters, or any evaluation of the output
  comes in, use `tb` or `rs` for that part.
- For the common cases use the precompiled `tb` first; it starts instantly
  because nothing is compiled: `tb json` (pretty-print, `-p /json/pointer`,
  `-k` keys or length; also JSON5, YAML, TOML), `tb conv FILE --to
  json|yaml|toml|csv`, `tb csv FILE` (columns, types, empty cells,
  min/max/mean, `-n` first rows), `tb xlsx FILE` (sheet as CSV, `-l` lists
  sheets), `tb count` (replaces `sort | uniq -c | sort -rn`, `-r` counts the
  first regex group) and `tb size DIR` (size and file count per entry).
  `tb <command> --help` shows all options. Searching text stays with `rg`.
- Structured data (JSON, CSV, XML, TOML, YAML) is always read with a parser in
  `tb` or `rs` (serde_json, csv, roxmltree, toml, serde_yaml_ng), never with
  `grep`/`sed`/`awk` or other regex matching, even for a quick look. Only
  `head`/`wc` to peek at raw bytes or line counts are fine.
- Use Python only where its libraries are clearly better: Word and PowerPoint,
  PDF of any kind (reading text, tables and form fields as well as creating
  and editing), editing existing Excel files, mail files (`.msg`, `.eml`), OCR,
  plots, machine learning and statistics, LLM SDKs, and Windows-specific work
  (COM, WMI, clipboard, GUI automation), which Rust scripts cannot do because
  they run in WSL (Linux). Reading Excel files or writing new ones stays with
  Rust (calamine only reads, rust_xlsxwriter only creates new files). Decide
  by library quality for the task, not by habit or by how short the script is.
- Walk or search large directory trees on the Windows side (thousands of
  files, for example the Desktop or a project with `node_modules`) with
  Windows-native tools such as `rg`, Git Bash `grep`/`find` or the Python
  venv, not with Rust in WSL: each file access across the boundary costs
  about 0.5 ms (see "File access across WSL and Windows").
- Use one language per task: when part of a small task needs Python, do the
  whole task in Python instead of splitting it across two scripts. Split only
  real data pipelines; Polars and Parquet exist on both sides, so data can be
  handed between a Python and a Rust step as Parquet or CSV files.

### Rust toolbox (WSL)

- `~/.rust-tools` in WSL (from Windows:
  `<WSL_HOME_UNC>\.rust-tools`) is a Cargo project whose
  dependencies are precompiled. The wrapper `rs` (`~/.local/bin/rs`) copies a
  single-file script into it, builds it with the `script` profile (release,
  incremental, mold linker) and runs it in the caller's working directory.
- Run in WSL: `rs script.rs [args]`, or `rs -e 'code'` for a snippet that becomes
  the body of `fn main() -> anyhow::Result<()>` (`Context`, `Result`, `bail` are
  imported). The wrapper appends `Ok(())` after the snippet itself, so do not
  end the snippet with `Ok(())`; that fails with a type-inference error
  (E0282). From Windows: `wsl.exe -e <WSL_HOME>/.local/bin/rs <script> [args]`;
  in Git Bash prefix `MSYS_NO_PATHCONV=1`, otherwise `/home/...` and `/mnt/...`
  arguments are rewritten to Windows paths. A Windows script path (`C:\...`) is
  converted, other arguments are passed unchanged, so give the script
  `/mnt/c/...` paths. `wsl.exe` refuses to start when the Windows working
  directory path is very long; `cd` to a short directory first. `tb` is called
  the same way: `wsl.exe -e <WSL_HOME>/.local/bin/tb <command> [args]`.
- Start time (measured): unchanged script 20–60 ms in WSL (about 0.2 s via
  `wsl.exe`); new or edited script about 1 s, with polars, calamine,
  rust_xlsxwriter or zip about 2.5–3.5 s. Image scripts take about 1 s when
  they load and save through `tb::img` (below), but about 14 s with
  `image::open` or `.save()`. Concurrent `rs` builds wait for each other (one
  cargo lock), so a parallel session can add waiting time. Only the precompile after changing
  `Cargo.toml` is slow (polars alone takes several minutes), so never
  interrupt a running `rs` build or `rs --warmup`.
- A script is a plain `fn main()` file with `use` statements for toolbox crates;
  it needs no `[dependencies]` header. Prefer `anyhow::Result` and `?`.
  Compiler messages point to the script's own path; warnings about unused
  items are switched off.
- Load and save images with the precompiled wrappers in the toolbox library
  `tb` (`~/.rust-tools/src/lib.rs`): `tb::img::open(path)`, `tb::img::load(&bytes)`,
  `tb::img::save(&dynamic_image, path)`, `tb::img::save_as(.., ImageFormat)`,
  `tb::img::encode(&img, ImageFormat) -> Vec<u8>`, and `tb::img::save_rgb`,
  `save_rgba`, `save_gray` for `RgbImage`/`RgbaImage`/`GrayImage`. Everything
  else (resize, crop, pixels, imageproc) uses `image`/`imageproc` directly.
- Do not write a crate API from memory when unsure of it; APIs change between
  versions (especially polars, image/imageproc, plotters, reqwest, calamine).
  Look up the exact version in `~/.rust-tools/Cargo.lock`, then read the
  examples, README and signatures in
  `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/` before writing
  the script.
- Follow "File access across WSL and Windows" (`rules/`): wrap every `File` in
  `BufWriter`/`BufReader` or use `fs::write`/`fs::read` once.
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
  Prefer pure-Rust crates with minimal features; changing `Cargo.toml` makes each
  script rebuild once on its next run. Add a wrapper to `src/lib.rs` only for a
  call whose first build was measured above about 3 s (generic code that each
  script would compile again); changing `src/lib.rs` relinks every script once.
  Changes to `src/bin/tb.rs` take effect after `rs --warmup`.
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
only copy of them: `running-commands.md` (estimate runtimes, run long commands
in the background, show progress) and `wsl-windows-files.md` (file access
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
