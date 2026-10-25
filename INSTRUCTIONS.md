# Agent instructions: ad-hoc scripting with Rust and Python

Copy everything below the line into the agent's global instruction file
(for Claude Code: `%USERPROFILE%\.claude\CLAUDE.md`; other agents: their
equivalent, such as `AGENTS.md`). Replace the placeholders with this machine's
values first:

| Placeholder | Meaning | Example |
|---|---|---|
| `<WIN_HOME>` | Windows user profile (`%USERPROFILE%`) | `C:\Users\<name>` |
| `<WIN_HOME_WSL>` | the same folder seen from WSL | `/mnt/c/Users/<name>` |
| `<WSL_HOME>` | home directory in WSL (`$HOME`) | `/home/<user>` |
| `<WSL_HOME_UNC>` | WSL home seen from Windows | `\\wsl.localhost\<distro>\home\<user>` |
| `<REPO>` | GitHub repository of this toolbox | `<owner>/<repo>` |

---

## Ad-hoc scripting: Rust and Python instead of PowerShell

- On Windows, do ad-hoc work (file and directory operations beyond the dedicated
  tools, data processing, Excel/PDF/Office/JSON handling, HTTP requests, system
  inspection) with one of the two preinstalled toolboxes below instead of
  PowerShell cmdlets, pipelines, or scripts. Use PowerShell only when neither
  fits. Invoking the toolboxes and native executables such as `git`, `gh`, or
  `cargo` directly from the shell is not affected by this rule.
- Write ad-hoc scripts to the temp directory (the session scratchpad when the
  harness provides one, otherwise `%TEMP%`), run them from there, and delete
  them as soon as they are no longer needed.
- Both toolboxes are for tooling only. Real projects keep their own `.venv/` or
  `Cargo.toml`; never add project dependencies to a toolbox.
- Both are backed up in the private repo `<REPO>` (checked out as
  `~/.rust-tools`; the Python list is copied to `python/requirements.txt`).
  After changing either toolbox, run `~/.rust-tools/backup.sh "<conventional
  commit title>"` in WSL. It commits directly to `main`; this backup-only repo
  needs no issue or pull request. When these instructions change, update
  `INSTRUCTIONS.md` in the repo as well.

### Choosing Rust or Python

- Rust is the default for everything, including one-off scripts, quick
  inspections and one-liners (`rs -e`), and for files on the Windows side
  (read them through `/mnt/c/...` with buffered or whole-file I/O). It compiles
  in 1–2.5 s, runs faster, and the compiler catches errors before a long run.
- Use Python only where its libraries are clearly better: Word and PowerPoint,
  PDF creation or editing, OCR, plots, machine learning and statistics, LLM
  SDKs, and Windows-specific work (COM, WMI, clipboard, GUI automation), which
  Rust scripts cannot do because they run in WSL (Linux). Decide by library
  quality for the task, not by habit or by how short the script is.
- Polars and Parquet exist on both sides, so data can be handed between a Python
  and a Rust step as Parquet or CSV files.

### Rust toolbox (WSL)

- `~/.rust-tools` in WSL (from Windows:
  `<WSL_HOME_UNC>\.rust-tools`) is a Cargo project whose
  dependencies are precompiled. The wrapper `rs` (`~/.local/bin/rs`) copies a
  single-file script into it, builds it with the `script` profile (release,
  incremental, mold linker) and runs it in the caller's working directory.
- Run in WSL: `rs script.rs [args]`, or `rs -e 'code'` for a snippet that becomes
  the body of `fn main() -> anyhow::Result<()>` (`Context`, `Result`, `bail` are
  imported). From Windows: `wsl.exe -e <WSL_HOME>/.local/bin/rs <script> [args]`;
  in Git Bash prefix `MSYS_NO_PATHCONV=1`, otherwise `/home/...` and `/mnt/...`
  arguments are rewritten to Windows paths. A Windows script path (`C:\...`) is
  converted, other arguments are passed unchanged, so give the script
  `/mnt/c/...` paths. `wsl.exe` refuses to start when the Windows working
  directory path is very long; `cd` to a short directory first.
- Start time (measured): unchanged script 20–60 ms in WSL (about 0.2 s via
  `wsl.exe`); new or edited script about 1 s, with polars about 1.5–2.5 s,
  with image and reqwest up to about 4 s. The first build of a script that
  combines plotters, geo and imageproc takes about 12 s; later edits of it
  about 1 s. Concurrent `rs` builds wait for each other (one cargo lock), so
  a parallel session can add waiting time. Only the precompile after changing
  `Cargo.toml` is slow (polars alone takes several minutes), so never
  interrupt a running `rs` build or `rs --warmup`.
- A script is a plain `fn main()` file with `use` statements for toolbox crates;
  it needs no `[dependencies]` header. Prefer `anyhow::Result` and `?`.
  Compiler messages point to the script's own path; warnings about unused
  items are switched off.
- Do not write a crate API from memory when unsure of it; APIs change between
  versions (especially polars, image/imageproc, plotters, reqwest, calamine).
  Look up the exact version in `~/.rust-tools/Cargo.lock`, then read the
  examples, README and signatures in
  `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/` before writing
  the script.
- Follow "File access across WSL and Windows" below: wrap every `File` in
  `BufWriter`/`BufReader` or use `fs::write`/`fs::read` once.
- Usable directly (full list with versions in `~/.rust-tools/Cargo.toml`):
  - Errors/CLI/output: anyhow, clap (derive), indicatif, comfy-table,
    owo-colors, humantime, bytesize
  - Formats: serde (derive), serde_json, serde_yaml_ng, toml, json5, quick-xml
    (serialize), roxmltree (easy XML reading), csv, dotenvy
  - Data/math: polars (lazy, csv, parquet, json, sql, strings, temporal),
    ndarray, statrs, rand, itertools, petgraph, rayon, geo (geometry and
    distances), plotters (simple PNG/SVG charts)
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
  script rebuild once on its next run.
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

## File access across WSL and Windows

This applies to every program, script, build and tool, in any language.

- Every file syscall that crosses between WSL and Windows costs about 0.5 ms:
  WSL programs on `/mnt/c/...`, and Windows programs on
  `\\wsl.localhost\...`. Many small reads or writes therefore become very slow;
  an unbuffered JPEG encoder made 135,731 writes to `/mnt/c` and took 75 s
  instead of 0.45 s.
- Always buffer file I/O there, or read and write whole files in one call:
  `BufWriter`/`BufReader` or `fs::write`/`fs::read` in Rust; in C/C++ avoid
  `write()` per item; in Python keep the default buffering (never
  `buffering=0`) and prefer `read_bytes`/`write_bytes`. Pass buffered writers to
  encoders and serializers instead of raw files.
- Keep many small working files (caches, downloaded tiles, intermediate
  outputs, build directories) on the side of the program that uses them: WSL
  programs under the WSL filesystem (for example `/tmp` or `~`), Windows
  programs under `C:\`. Cross over only for inputs and final results.
- Build and run projects on the filesystem of the toolchain that builds them
  (`cargo`, `git`, `npm` in WSL on WSL paths). For a project that lives under
  `C:\`, prefer the Windows toolchain or expect slow builds.
- If a run is unexpectedly slow, check the syscall count first
  (`strace -c -f <cmd>` in WSL) before suspecting the network or the
  compiler.
