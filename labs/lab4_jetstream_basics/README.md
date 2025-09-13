


# Lab 4 – JetStream Basics

This lab introduces **JetStream**, NATS’s persistence layer that adds at-least-once delivery, streams, and consumers. You will create a stream, publish messages into it, and then consume them with a pull-based consumer that requires explicit acknowledgements.

## Purpose

- Learn the difference between Core NATS (ephemeral, fire-and-forget) and JetStream (persistent).
- Create a **Stream** that stores messages for a subject.
- Publish messages into the stream.
- Create a **Consumer** and pull messages with explicit acks.

## What you use

- Lab-specific executables:
  - `js-bootstrap` — creates a stream and consumer for the lab.
  - `js-pub` — publishes messages into the stream.
  - `js-pull` — pulls messages from the consumer and acks them.
  - `js-purge` — purges stream messages.
- Config files:
  - Root: `config.toml` (required)
  - Lab: `labs/lab4_jetstream_basics/config.toml` (stream/subject/consumer overrides)

## Prerequisites

- Local NATS server running with JetStream enabled (Docker Compose `nats -js`).
- Root `config.toml`:
  ```toml
  [runtime]
  mode = "local"

  [nats]
  url = "nats://127.0.0.1:4222"

  [recv]
  wait_secs = 5
  ```
- Lab config at `labs/lab4_jetstream_basics/config.toml`:
  ```toml
  [nats]
  subject = "lab4.stream"
  stream  = "LAB4_STREAM"
  consumer = "LAB4_CONSUMER"
  ```

## Commands

### Bootstrap stream and consumer
```bash
make LAB=lab4_jetstream_basics run BIN=js-bootstrap
```

### Publish messages into the stream
```bash
make LAB=lab4_jetstream_basics run BIN=js-pub MSG="first message"
make LAB=lab4_jetstream_basics run BIN=js-pub MSG="second message"
```

### Pull and ack messages
```bash
make LAB=lab4_jetstream_basics run BIN=js-pull
```

You should see:
```
INFO pulled message: "first message"
INFO acked
INFO pulled message: "second message"
INFO acked
```

### Purge the stream
```bash
make LAB=lab4_jetstream_basics run BIN=js-purge
```

## Expected output

- Messages are persisted in the stream and delivered at least once.
- Consumers must **ack** each message; otherwise, the server will redeliver after the ack wait expires.

## Key takeaways

- Core NATS is ephemeral; JetStream introduces **streams** and **consumers**.
- Messages are persisted in streams bound to subjects.
- Consumers (durable or ephemeral) control how messages are delivered.
- Acks are required to signal successful processing.

## Technical details

- Stream creation specifies name + subjects:
  ```rust
  let js = client.jetstream().await?;
  js.create_or_update_stream(stream::Config {
      name: stream_name.clone(),
      subjects: vec![subject.clone()],
      ..Default::default()
  })
  .await?;
  ```

- - Publishing into JetStream is two-stage: first `publish` returns an ack future, then await it to confirm persistence:
  ```rust
  let ack_fut = js.publish(subject.clone(), body.into()).await?;
  ack_fut.await.map_err(|e| anyhow!("publish ack failed: {e}"))?;
  ```

- Pulling messages requires acking
  ```rust
  let stream = js
    .get_stream(stream_name)
    .await
    .map_err(|e| anyhow!("failed to get stream: {e}"))?;

  let consumer: consumer::PullConsumer = stream
    .get_consumer(&consumer_name)
    .await
    .map_err(|e| anyhow!("failed to get consumer: {e}"))?;

  while let Some(Ok(msg)) = consumer.messages().await?.next().await {
    println!("got {:?}", msg);
    msg.ack().await?;
  }
  ```

## Common misunderstandings

- “JetStream guarantees exactly-once.” → No, JetStream is **at-least-once**. You must make processing idempotent.
- “Streams exist automatically.” → No, you must create a stream first.
- “Consumers automatically ack.” → No, you must explicitly call `ack()`.

## Cleanup

Purge or delete the stream if you want to reset state:
```bash
make LAB=lab4_jetstream_basics run BIN=js-purge
```

## Repo layout (relevant parts)

```
/labs
  /lab4_jetstream_basics
    README.md
    config.toml
    src/bin/js-bootstrap.rs
    src/bin/js-pub.rs
    src/bin/js-pull.rs
    src/bin/js-purge.rs
```