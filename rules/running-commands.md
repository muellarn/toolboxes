## Efficient scripts

- **Stream, do not load everything.** Read line-delimited data (JSONL, CSV) with
  a reader per record instead of `read_to_string` plus a full parse. Only load a
  whole file when it is small or genuinely needs random access.
- **Build one compact intermediate and reuse it.** Do not re-parse the same
  large file in every analysis script. Convert once into a narrow format
  (Parquet, or CSV with only the needed columns) and read that afterwards.
- **Keep only the columns and fields the next step needs.** A 150 MB JSON that
  carries fields nobody reads is 150 MB of I/O and parsing per run.
- **Do not re-fetch what is already on disk.** Check for the file first and skip
  the request; make harvests resumable by appending and deduplicating.
- **Parallelize network work against the real rate limit.** Match the pool size
  to the documented limit (SEC allows 10 requests per second) and add retries
  with backoff instead of aborting the whole run on one failure.
- **Write results incrementally and flush per batch**, so a crash costs one batch
  rather than the run.
- **Measure before optimizing.** If a run is slow, find out whether it is I/O,
  parsing, network or CPU before changing anything.

## Running commands: estimate, background, report status

This applies to every command and script, in any language, foreground or
background.

- **Every command gets a timeout of 5 seconds.** A command that should take
  longer is announced first: one sentence on how long it should take and what
  drives that (number of requests, files, rows, build time). It then gets a
  matching timeout: close to the estimate, not padded far beyond it, or run in
  the background so that the wait can be extended when the estimate was wrong.
- **Anything that runs longer than 5 seconds logs what it does.** No command
  may run that long without feedback on what is happening. If the command
  cannot report its progress inline, run it through a script (Rust, Python, sh
  or PowerShell) that does.
- **Long runs go to the background by default.** If the estimate is more than
  roughly 30 seconds, start the command in the background without being asked:
  with the harness's background mode if it has one, otherwise detached with
  its output in a log file (`nohup cmd > run.log 2>&1 &`), and read the log
  instead of waiting on the command.
- **Always produce visible progress.** Scripts print a counter, a percentage or
  a heartbeat every few seconds to stderr; tools get their progress or verbose
  flag. A silent run cannot be told apart from a hang.
- **Tell the owner the status.** While a command runs, report what it is doing
  and how far it is, not only the final result.
- **Check the estimate against reality.** At reasonable intervals compare the
  elapsed time and the progress with the estimate. If the run is clearly slower
  (for example twice the estimate), check whether it still makes progress and
  whether continuing makes sense: reduce the workload, change the approach or
  stop it. Do not let a run drift silently past its estimate. Exception: never
  interrupt a running `rs` build, `rs --warmup` or another build that leaves
  broken artifacts; report the delay instead.
- **Report large deviations** to the owner together with what is being done
  about them.
- **Keep queues short.** Do not start a second long job while a slow one is
  still running unless they are independent and the machine has room for both.
  Concurrent `rs` builds wait for each other, so parallel runs only help for
  work that is already compiled.
- **After the run, state the actual duration** next to the estimate, so the
  next estimate is better.
