# Lab 1 – NATS “Hello Subject”: Publish / Subscribe (Core NATS)

Minimal Core NATS flow in Rust: **publish** to a subject and **subscribe** to it.  
This lab highlights that Core NATS is **fire-and-forget** (no persistence, no redelivery) — if a subscriber isn’t online at publish time, the message is **lost**. JetStream (persistence/acks) comes later.

## Purpose

- Connect to a local NATS server.
- **Publish** a message to a subject (e.g., `lab1.hello`).
- **Subscribe** and print messages in real time.
- See how subjects work (simple, hierarchical names), and why this is **not a queue** (no storage, no redelivery).

## What you use

- **Shared executables** (in `shared`):
  - `nats-sub` — simple subscriber: connects, subscribes to one subject, prints messages.
  - `nats-pub` — simple publisher: connects and publishes one message.
- **Config files**:
  - Root config (required): `config.toml`
  - Lab config (optional overrides): `labs/lab1_publish_subscribe/config.toml`
- **Docker Compose**: local `nats-server` with JetStream enabled (JetStream isn’t used in this lab, but the server supports it).

## Prerequisites

- Rust (stable) + Cargo
- `make`
- Docker (for local server via `docker-compose`)
- Root **`config.toml`** (required):

```toml
[runtime]
mode = "local"

[nats]
# Local server (docker-compose). Change to your remote URL if needed.
url = "nats://127.0.0.1:4222"

[recv]
wait_secs = 5   # subscriber sleep/backoff, used by shared tools where relevant
```

- Lab 1 config (subject for this lab): `labs/lab1_publish_subscribe/config.toml`

```toml
[nats]
subject = "lab1.hello"
```

> Config precedence: **root** → **lab** → `APP_` environment overrides. Root config must exist (the Makefile guards it).  

## Start the server (optional, for local runs)

```bash
make up
```

This brings up `nats-server` on `nats://127.0.0.1:4222`.

- To stop: `make down`

## Commands (run from repo root)

### 1) Start the subscriber (Terminal A)

```bash
make LAB=lab1_publish_subscribe sub
```

You should see something like:

```
YYYY-MM-ddT10:03:02.627505Z  INFO nats_sub: subscriber starting url=nats://127.0.0.1:4222 subject=lab1.hello
YYYY-MM-ddT10:03:02.633548Z  INFO async_nats::connector: connected successfully server=4222 max_payload=1048576
YYYY-MM-ddT10:03:02.633987Z  INFO nats_sub: waiting for messages (Ctrl+C to stop)
YYYY-MM-ddT10:03:02.634072Z  INFO async_nats: event: connected
```

> To override the subject without changing TOML:  
> `make LAB=lab1_publish_subscribe sub ARGS="--subject other.subject"`

### 2) Publish a message (Terminal B)

```bash
make LAB=lab1_publish_subscribe pub MSG="hello world"
```

Expected:

```
YYYY-MM-ddT10:05:37.563867Z  INFO nats_pub: publishing message url=nats://127.0.0.1:4222 subject=lab1.hello body=hello world
YYYY-MM-ddT10:05:37.569270Z  INFO async_nats::connector: connected successfully server=4222 max_payload=1048576
YYYY-MM-ddT10:05:37.569702Z  INFO async_nats: event: connected
YYYY-MM-ddT10:05:37.569818Z  INFO nats_pub: message published subject=lab1.hello
```

Subscriber prints:

```
[sub] msg on lab1.hello: "hello world"
```

> To override the subject at publish time:  
> `make LAB=lab1_publish_subscribe pub ARGS="--subject lab1.hello" MSG="hi again"`

## Behavior & Expected Output

- **Subscriber running first:** Every `pub` is printed by the subscriber.
- **Subscriber not running:** Messages are **dropped** (Core NATS does not persist). When you start the subscriber later, it only sees *new* messages.
- **Multiple subscribers (no queue groups):** All active subscribers on the same subject receive a **copy** (fan-out).
- **Subjects are hierarchical strings** (e.g., `orders.us.created`). Wildcards (`*`, `>`) are supported, but we keep a single subject in this lab.

## Key Takeaways

- **Core NATS is fire-and-forget**: no persistence, no redelivery, no offsets.
- **Subscribers must be online** to receive messages.
- **Subjects** are lightweight routing keys; multiple subscribers on the same subject all receive the message (pub/sub fan-out).
- **This is not a queue** (no competing consumers here). Competing consumers in Core NATS use **queue groups** (next lab). Persistence and “at-least-once” delivery require **JetStream** (later lab).

## Common Misunderstandings

- “If I publish and then start a subscriber, it will catch up.” → **False** in Core NATS. You need JetStream to replay.
- “NATS guarantees delivery.” → Core NATS provides **best-effort** delivery. Network partitions or slow consumers can drop messages.
- “Subjects are like Kafka topics with partitions.” → Different model. Subjects are routing keys; there’s no partitioning or offset tracking in Core NATS.

## Cleanup

Nothing persistent was created in this lab. If you ran the local server:

```bash
make down
```

## Repo layout (relevant parts)

```
Makefile
config.toml                 # root (required)

/shared
  src/bin/
    nats-sub.rs             # shared subscriber
    nats-pub.rs             # shared publisher

/labs
  /lab1_publish_subscribe
    README.md
    config.toml             # per-lab subject
```
