# ops/

Unattended queue runner and operational helpers for PostCAD lane-1 campaigns.

---

## Queue Runner

Run all pending campaigns:

```bash
bash ops/queue_ctl.sh start
```

Run at most N campaigns:

```bash
bash ops/queue_ctl.sh start --max 3
```

Check queue state (phone-safe):

```bash
bash ops/queue_ctl.sh status
bash ops/queue_ctl.sh pending
bash ops/queue_ctl.sh tail
```

---

## Notification Hook

`ops/notify.sh` is a best-effort notifier called by the queue runner on important events. It always appends a formatted line to `ops/logs/notify.log` and prints to stderr. It never fails the queue.

### Default (local log only)

No configuration needed. Events are written to `ops/logs/notify.log`:

```
[2026-03-16T20:00:00Z] [queue-finished] queue — PASSED — 3 passed, 0 blocked
[2026-03-16T20:00:01Z] [campaign-failed] my-campaign — failed after 2 attempts — see ...
[2026-03-16T20:00:02Z] [guard-blocked] my-campaign — KERNEL PATH FORBIDDEN: crates/core
```

### Optional external notifier

Set `POSTCAD_NOTIFY_CMD` to any shell command or script path. It will be called with three positional arguments: `<event> <campaign> <message>`.

```bash
export POSTCAD_NOTIFY_CMD="/path/to/my-notifier.sh"
bash ops/queue_ctl.sh start
```

Example — log to a file with a custom prefix:

```bash
export POSTCAD_NOTIFY_CMD="echo POSTCAD-ALERT:"
bash ops/queue_ctl.sh start
# emits: POSTCAD-ALERT: queue-finished queue PASSED — 2 passed, 0 blocked
```

### Notified events

| Event            | When                                              |
|------------------|---------------------------------------------------|
| `guard-blocked`  | Campaign file touches forbidden or out-of-lane path |
| `campaign-failed`| Campaign fails both execution attempts            |
| `queue-finished` | Queue reaches terminal state (PASSED/PARTIAL/BLOCKED) |

Success of individual campaigns does not generate a notification.

---

## Files

| Path                          | Purpose                                      |
|-------------------------------|----------------------------------------------|
| `ops/run_campaign_queue.sh`   | Unattended queue runner                      |
| `ops/queue_ctl.sh`            | Phone-friendly queue control shortcuts       |
| `ops/notify.sh`               | Best-effort notification helper              |
| `ops/campaign_queue/`         | Pending campaign files (`.md`)               |
| `ops/campaign_queue/done/`    | Completed campaigns                          |
| `ops/campaign_queue/logs/`    | Per-campaign execution logs                  |
| `ops/logs/notify.log`         | Notification event log                       |
| `ops/queue_status.log`        | Queue status event log                       |
| `ops/last_result.md`          | Latest queue run summary (morning dashboard) |
| `ops/setup_two_worker_fleet.sh` | Bootstraps 2-worker isolated worktree layout |
| `ops/worker_fleet_status.sh`    | Read-only status inspector for the 2-worker fleet |

---

## Two-Worker Fleet

For running two independent lane-1 campaigns in parallel without interference,
the repo supports a 2-worker isolated worktree model.

### Why worktrees

Each worker is a `git worktree` — a full checkout of the repo on a dedicated
branch (`worker/w1`, `worker/w2`), rooted outside the main repo directory.
Worktrees share the object store but have independent working trees and HEAD
pointers, so each worker's in-progress changes are fully isolated from the
other and from `main`.

### Bootstrap

Run once to create both worker directories and branches:

```bash
bash ops/setup_two_worker_fleet.sh
```

Default layout:

```
~/workers/postcad-w1   (branch: worker/w1)
~/workers/postcad-w2   (branch: worker/w2)
```

Use `--base-dir` to choose a different root:

```bash
bash ops/setup_two_worker_fleet.sh --base-dir /path/to/workers
```

The script is idempotent: if a worktree already exists and is correctly
registered it is reused without modification.

### Check fleet status

After bootstrap, inspect both workers at any time with:

```bash
bash ops/worker_fleet_status.sh
```

Or with a custom base directory:

```bash
bash ops/worker_fleet_status.sh --base-dir /path/to/workers
```

Prints sections: `REPO`, `BASE DIR`, `WORKER A`, `WORKER B` (path, registration
status, branch, clean/dirty), `claude` availability, and a `NEXT` section with
manual entry commands or a bootstrap reminder if workers are missing. Read-only.
No git state is modified.

Typical bootstrap → status → entry flow:

```bash
bash ops/setup_two_worker_fleet.sh        # create worktrees
bash ops/worker_fleet_status.sh           # verify state
cd ~/workers/postcad-worker-a && claude   # enter worker A manually
```

### Entering a worker and running Claude

```bash
cd ~/workers/postcad-w1
claude
```

```bash
cd ~/workers/postcad-w2
claude
```

Paste the campaign prompt into Claude after launching. Nothing starts
automatically.

### Safety rules

| Rule | Detail |
|------|--------|
| One campaign per worker | Never assign two campaigns to the same worker concurrently |
| No file overlap | Two concurrent campaigns must not touch the same allowed files |
| Kernel files off-limits | `crates/core`, `crates/routing`, `crates/compliance`, `crates/audit`, `crates/registry` are never modified in workers |
| No auto-merge | Founder reviews each worker branch and merges to `main` manually |
| No auto-push | Push only after the campaign is complete, tested, and reviewed |
| No auto-start | Claude is launched manually inside each worker; there is no daemon or queue runner in workers |
