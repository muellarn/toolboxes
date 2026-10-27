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
- **Default working directory for WSL scripts is `/tmp`, not `/mnt/c`.**
  Write inputs, caches, intermediate files and logs under `/tmp` (or `~`), and
  copy only the final result to the Windows side. Do not run a WSL script whose
  data lives on `/mnt/c` just because it is convenient.
- **`/tmp` is wiped by a WSL restart, so protect the result.** Write the final
  output to `/tmp` first and copy it to `/mnt/c` as soon as it exists, or write
  it directly to `/mnt/c` in one call. Never leave the only copy of hours of
  work in `/tmp`. Long harvests append to a file and flush per batch, so a
  crash costs one batch, not the run.
- **Avoid large JSON as an interchange format.** A 150 MB JSON costs seconds of
  I/O plus seconds of parsing on every read. Prefer Parquet, CSV or a compact
  binary layout for anything above a few megabytes.
- Build and run projects on the filesystem of the toolchain that builds them
  (`cargo`, `git`, `npm` in WSL on WSL paths). For a project that lives under
  `C:\`, prefer the Windows toolchain or expect slow builds.
- If a run is unexpectedly slow, check the syscall count first
  (`strace -c -f <cmd>` in WSL) before suspecting the network or the
  compiler.
