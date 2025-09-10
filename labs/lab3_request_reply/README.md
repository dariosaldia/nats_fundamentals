# Lab 3 – Request / Reply

This lab demonstrates the NATS **request/reply** pattern. A client sends a request message and waits for a reply. The server (responder) receives the request on a subject and replies to the inbox subject provided by the client.

## Purpose

- Learn how NATS supports request/reply interactions.
- Implement a simple responder that listens on a subject and replies.
- Implement a requester that sends a request and waits for a reply with a timeout.

## What you use

- Lab-specific executables:
  - `responder` — listens for requests and replies.
  - `requester` — sends a request and waits for reply.
- Config files:
  - Root: `config.toml` (required)
  - Lab: `labs/lab3_request_reply/config.toml` (subject override)

## Prerequisites

- Local NATS server running.
- Root `config.toml`:
  ```toml
  [runtime]
  mode = "local"

  [nats]
  url = "nats://127.0.0.1:4222"

  [recv]
  wait_secs = 5
  ```
- Lab config at `labs/lab3_request_reply/config.toml`:
  ```toml
  [nats]
  subject = "lab3.rpc"
  ```

## Commands

### Start the responder
```bash
make LAB=lab3_request_reply run BIN=responder
```

### Send a request
```bash
make LAB=lab3_request_reply run BIN=requester ARGS="--msg 'hello server'"
```

## Expected output

Responder:
```
INFO responder listening on subject=lab3.rpc
INFO received request: "hello server"
INFO sent reply: "ack: hello server"
```

Requester:
```
INFO sent request to subject=lab3.rpc
INFO received reply: "ack: hello server"
```

## Technical details

This lab uses NATS’s built‑in request/reply.

- **Responder** subscribes to the request subject and replies to the `reply` inbox carried on each message. A `flush()` ensures the reply is sent before looping.
  ```rust
  let client = async_nats::connect(&url).await?;
  let mut sub = client.subscribe(subject.clone()).await?;
  
  while let Some(msg) = sub.next().await {
      let req = String::from_utf8_lossy(&msg.payload);
      let reply = format!("ack: {}", req);
      if let Some(reply_to) = msg.reply {
          client.publish(reply_to, reply.into()).await?;
          client.flush().await?; // ensure the reply hits the wire
      }
  }
  ```

- **Requester** sends a request and waits for the single reply with a timeout. Under the hood, the client uses an **inbox subject** for the reply.
  ```rust
  use tokio::time::{timeout, Duration};
  
  let client = async_nats::connect(&url).await?;
  let fut = client.request(subject.clone(), body.clone().into());
  let resp_msg = timeout(Duration::from_millis(timeout_ms), fut).await??;
  let resp = String::from_utf8_lossy(&resp_msg.payload);
  ```

Notes:
- Core NATS doesn’t persist requests or replies. If the responder isn’t running, the requester times out.
- Multiple responders on the same subject may race to reply; the requester returns the **first** reply it receives.

## Key takeaways

- Request/reply is built-in to NATS using **inbox subjects** under the hood.
- The client sends a request and waits for the reply with a timeout.
- The server replies to the inbox subject carried with the request message.
- This pattern is useful for RPC-style interactions.

## Common misunderstandings

- “NATS request/reply stores messages.” → No. If the responder is not online, the request times out. There is no persistence in Core NATS.
- “One request can fan out to many replies.” → By default, only one reply is expected. Multiple responders on the same subject may cause multiple replies, which the client should be prepared to handle.

## Cleanup

Stop responder and requester processes (Ctrl+C). No persistent resources are created.

## Repo layout (relevant parts)

```
/labs
  /lab3_request_reply
    README.md
    config.toml
    src/bin/responder.rs
    src/bin/requester.rs
```