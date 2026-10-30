## File access across WSL and Windows

This applies to every program, script, build, and tool, in any language.

- Every file syscall that crosses between WSL and Windows costs about 0.5 ms:
  WSL programs on `/mnt/c/...` and Windows programs on `\\wsl.localhost\...`.
  Many small reads or writes therefore become over 100 times slower.
- Buffer all file I/O there, or read and write whole files in one call:
  `BufReader`/`BufWriter` or `fs::read`/`fs::write` in Rust, no `write()` per
  item in C/C++, default buffering (never `buffering=0`) and
  `read_bytes`/`write_bytes` in Python. Pass buffered writers to encoders and
  serializers instead of raw files.
- Keep working files (caches, downloads, intermediate outputs, logs, build
  directories) on the side of the program that uses them: under `/tmp` or `~`
  for WSL programs, under `C:\` for Windows programs. Cross the boundary only
  for inputs and final results.
- `/tmp` is wiped when WSL restarts: copy a final result to `/mnt/c` as soon as
  it exists, or write it there directly in one call. Never leave the only copy
  of hours of work in `/tmp`.
- Build and run projects with the toolchain of the filesystem they live on:
  `cargo`, `git`, and `npm` in WSL for WSL paths; for a project under `C:\`,
  prefer the Windows toolchain or expect slow builds.
- If a run is unexpectedly slow, check the syscall count first
  (`strace -c -f <cmd>` in WSL).
