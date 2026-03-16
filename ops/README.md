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
