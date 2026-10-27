## Running commands: estimate, background, report status

This applies to every command and script, in any language, foreground or
background.

- **Estimate before starting.** Say in one line how long the command should
  take and what drives that (number of requests, files, rows, build time).
- **Long runs go to the background by default.** If the estimate is more than
  roughly 30 seconds, start the command in the background without being asked:
  with the harness's background mode if it has one, otherwise detached with
  its output in a log file (`nohup cmd > run.log 2>&1 &`), and read the log
  instead of waiting on the command.
- **Always produce visible progress.** Never start a command that can run for
  more than a few seconds without feedback. Scripts print a counter, a
  percentage or a heartbeat every few seconds to stderr; tools get their
  progress or verbose flag. A silent run cannot be told apart from a hang.
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
