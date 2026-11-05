## Efficient scripts

- Stream line-delimited data (JSONL, CSV) record by record; load a whole file
  only when it is small or needs random access.
- Convert a large input once into a compact intermediate that keeps only the
  needed columns (Parquet, or CSV) and reuse it instead of re-parsing the
  original in every script. Avoid large JSON as an interchange format.
- Do not re-fetch what is already on disk; make harvests resumable by
  appending and deduplicating.
- Parallelize network work up to the service's documented rate limit, with
  retries and backoff instead of aborting the run on one failure.
- Write results incrementally and flush per batch, so a crash costs one batch,
  not the run.
- Measure before optimizing: find out whether a slow run is bound by I/O,
  parsing, network, or CPU before changing anything.

## Running commands: estimate, background, report status

This applies to every command and script, in any language, foreground or
background.

- The default timeout is 5 seconds. Before a longer command, say in one
  sentence how long it should take and what drives that (requests, files, rows,
  build time), then give it a matching timeout, not padded far beyond the
  estimate, or run it in the background so the wait can be extended.
- Never give the 5-second default to `rs` builds, `cargo`, the first `wsl.exe`
  call of a session (WSL may have to start), or network commands (`git fetch`
  or `push`, `gh`, downloads); give them a realistic timeout.
- After a timeout, do not rerun with the same timeout. First check that no
  leftover process or lock remains (a killed `wsl.exe` may leave its Linux
  process running, for example holding the cargo lock), then rerun with a
  realistic timeout or in the background.
- Run anything estimated above about 30 seconds in the background without being
  asked: with the harness's background mode if it has one, otherwise detached
  (`nohup ... &`). Its output goes to a log file framed by markers that carry
  the start time and the exit code:
  `{ echo "__START $(date +%s)"; cmd; echo "__EXIT $?"; } > run.log 2>&1`
  (PowerShell: `"__START $([DateTimeOffset]::Now.ToUnixTimeSeconds())"` and
  `"__EXIT $LASTEXITCODE"`).
- Watch every background run with `tb watch run.log` as the event source of the
  harness's monitor mechanism (Claude Code: the `Monitor` tool with a
  30-minute timeout), never with sleep or polling loops. From Windows, run
  `wsl.exe -e <WSL_HOME>/.local/bin/tb watch /mnt/c/.../run.log`. It reads the
  log every second at no cost and prints a line only for events: a status
  every 30 seconds (`--every`), alert lines such as errors, panics, or failures
  (`--alert` sets the regex), a warning when the log has not grown for 2
  minutes (`--stall`), and the end with exit code and duration. It stops after
  29 minutes (`--for`); re-arm it while the run continues.
- At every event, check that the run still does what the owner asked for and
  progresses at the estimated rate, and tell the owner in one line what it is
  doing and how far it is. If it is off (wrong work, errors, no progress, or
  clearly slower, for example twice the estimate), decide whether to reduce the
  workload, change the approach, or stop it, and report the deviation together
  with what is being done about it. Never interrupt a running `rs` build,
  `rs --warmup`, or another build that leaves broken artifacts; report the
  delay instead.
- A background run shows progress in its log: a counter, percentage, or
  heartbeat line every 10 to 30 seconds, or the tool's progress or verbose
  flag. If a command cannot report progress itself, run it through a script
  that does. A silent run cannot be told apart from a hang. Bulky output stays
  in the log; read only its tail.
- Do not start a second long job while a slow one is running unless they are
  independent and the machine has room for both. Concurrent `rs` builds wait
  for each other.
- Measure the duration (from the `__START` and `__EXIT` markers, `time`, or
  `Measure-Command`) and state it next to the estimate.
