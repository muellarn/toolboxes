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
