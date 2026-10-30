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
- Run anything estimated above about 30 seconds in the background without being
  asked: with the harness's background mode if it has one, otherwise detached
  with its output in a log file (`nohup cmd > run.log 2>&1 &`). Read the log
  instead of waiting on the command.
- Anything that runs longer than 5 seconds shows visible progress: a counter,
  percentage, or heartbeat every few seconds on stderr, or the tool's progress
  or verbose flag. If a command cannot report progress itself, run it through a
  script that does. A silent run cannot be told apart from a hang.
- While a command runs, tell the owner what it is doing and how far it is, not
  only the final result.
- At reasonable intervals compare elapsed time and progress with the estimate.
  If the run is clearly slower (for example twice the estimate), check that it
  still makes progress and decide whether to reduce the workload, change the
  approach, or stop it. Report large deviations together with what is being
  done about them. Never interrupt a running `rs` build, `rs --warmup`, or
  another build that leaves broken artifacts; report the delay instead.
- Do not start a second long job while a slow one is running unless they are
  independent and the machine has room for both. Concurrent `rs` builds wait
  for each other.
- After the run, state the actual duration next to the estimate.
