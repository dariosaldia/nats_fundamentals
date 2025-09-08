

# Lab 2 – Queue Groups (Competing Consumers)

Use NATS **queue groups** so multiple subscribers on the same subject share the load: each message is delivered to **one** worker in the group.

## Purpose

- Start multiple subscribers in the **same queue group**.
- Observe **load balancing**: one message → delivered to one worker.
- Contrast with simple pub/sub fan-out (Lab 1), where all subscribers receive a copy.

## What you use

- Shared executable: `nats-pub` (from `shared`)
- Lab-specific executable: `worker` (queue group subscriber)
- Config files:
  - Root: `config.toml` (required)
  - Lab: `labs/lab2_queue_groups/config.toml` (subject overrides)

## Prerequisites

- Local NATS server running (e.g., `make up` if you set up docker-compose)
- Root `config.toml`:
  ```toml
  [runtime]
  mode = "local"

  [nats]
  url = "nats://127.0.0.1:4222"

  [recv]
  wait_secs = 5
  ```
- Lab config at `labs/lab2_queue_groups/config.toml`:
  ```toml
  [nats]
  subject = "lab2.work"
  ```

## Commands

### Start two or more workers (same queue group)
```bash
make LAB=lab2_queue_groups run BIN=worker ARGS="--label A"
make LAB=lab2_queue_groups run BIN=worker ARGS="--label B"
make LAB=lab2_queue_groups run BIN=worker ARGS="--label C"
```

### Publish several messages
Use the shared publisher (flush is built in):
```bash
make LAB=lab2_queue_groups pub MSG="task-1"
make LAB=lab2_queue_groups pub MSG="task-2"
make LAB=lab2_queue_groups pub MSG="task-3"
```

## Expected output

Workers print messages they **alone** receive. With two workers (A and B), you should see load-balancing like:

Worker A:
```
INFO worker starting (queue group)
INFO waiting for messages…
INFO processed subject=lab2.work body="task-1" label=A
INFO processed subject=lab2.work body="task-3" label=A
```

Worker B:
```
INFO worker starting (queue group)
INFO waiting for messages…
INFO processed subject=lab2.work body="task-2" label=B
```


## Key takeaways

- Subscribers in the **same queue group** compete: each message is delivered to **one** of them.
- Different **queue group names** receive independent copies (fan-out across groups).
- Ordering is **not guaranteed** across multiple workers; if strict order matters, you’ll handle it at the application level or use JetStream with single consumer semantics later.

## Technical details

This lab introduces NATS **queue groups** (competing consumers). Instead of every subscriber getting a copy, all subscribers that join the *same* group share the work; each message is delivered to **one** member of that group.

**How it’s coded:**

- Join a queue group by calling `queue_subscribe` (not `subscribe`):
  ```rust
  let mut sub = client
      .queue_subscribe(subject.clone(), queue.clone())
      .await?;
  ```

- Core NATS has **no ack/persistence** here: if there are no active workers in the group, messages are **dropped**. Load balancing happens only among live members of the **same** `queue`.

## Common misunderstandings

- “Queue groups persist messages.” → No. This is still **Core NATS** (no persistence). If no worker is online, messages are dropped.
- “Queue group = one global subscriber.” → The set of workers with the **same group name** acts like one logical consumer from the publisher’s perspective.

## Cleanup

Stop your worker processes (Ctrl+C). Nothing persistent was created in this lab.

## Repo layout (relevant parts)

```
/shared
  src/bin/nats-pub.rs
/labs
  /lab2_queue_groups
    README.md
    config.toml
    src/bin/worker.rs
```