# PostCAD Night Mode

## Purpose

Night mode is for unattended lane-1 campaigns only.

## Allowed Scope

- `examples/pilot/**`
- `ops/**`
- `docs/**`
- `crates/service/tests/*surface_tests.rs`

## Stop Conditions

- forbidden file access
- ambiguous instruction
- failing test command
- missing required fixture

## Launch

Start the night queue in a detached tmux session:

```bash
bash ops/start_night_queue.sh
```

The session is named `postcad-night`. The command refuses to start if the
session already exists, so it is safe to call repeatedly from a scheduler.

**Inspect logs**

```bash
tail -f ops/logs/*.log
# or attach to the live session:
tmux attach -t postcad-night
```

**Check session and queue status**

Run the compact status summary at any time (safe when the session is absent):

```bash
bash ops/night_status.sh
```

Prints session state (`RUNNING` / `STOPPED`), the last few lines of
`ops/queue_status.log`, and the last few lines of `ops/last_result.md`.

**Tail the queue log (one command)**

`ops/night_tail.sh` is a focused, read-only helper for night-mode monitoring.
It tails `ops/queue_status.log` with an optional bounded line count.

```bash
# Default (20 lines)
bash ops/night_tail.sh

# Override line count
bash ops/night_tail.sh --lines 50
```

Behaviour:
- Refuses to run outside the PostCAD repository (checks `CLAUDE.md` marker).
- Exits non-zero with a clear message if `ops/queue_status.log` is missing.
- No writes, no network, no tmux management — safe to run at any time.

**Stop the session**

```bash
tmux kill-session -t postcad-night
```
