God please bless this work. In Jesus name.
Jesus is Lord and Savior. Jesus loves you.

# AI Chromium Router

AI Chromium Router is a local browser-control server that exposes an OpenAI-compatible API over authenticated browser sessions.

Its purpose is to let one API route work across:

- ChatGPT private chats
- ChatGPT group chats
- ChatGPT Projects
- ChatGPT Projects artifact upload
- Gemini chats
- Gemini file upload when available

The project is not mainly a scraper. It is a provider-capability router.

```text
request
  → provider selection
  → capability plan
  → browser action
  → evidence capture
  → learning loop
  → response extraction
  → replay/privacy verification
  → OpenAI-compatible response
```

## Core Equation

```text
Good = max(provider_coverage, capability_correctness, learning_rate, replay_determinism, privacy_safety, test_coverage, operator_clarity)
```

One-line explanation: the project becomes good when each browser capability is proven by action receipts, learning records, replayable evidence, privacy gates, and tests.

## Learning

Learning means the router converts successful browser turns into better future behavior.

```text
observe evidence → derive schemas → score actions → promote rules → generate extractors → replay verify
```

Nothing is learned directly from private content. Only redacted evidence, schemas, action receipts, and replay-passing rules may be promoted.

## Scope

The router should support these capability lanes:

| Surface           | Required capability                                                    |
|-------------------+------------------------------------------------------------------------|
| `chatgpt_private` | send prompt, read assistant response, attach files when supported      |
| `chatgpt_group`   | send group message, distinguish assistant response from human messages |
| `chatgpt_project` | select project, open/create chat, upload artifact, send prompt         |
| `gemini_private`  | send prompt, read response, upload files when supported                |

## Boundary

The browser session is operator-owned and already authenticated.

The router must not harvest credentials, bypass login, bypass CAPTCHA, evade rate limits, or persist cookies/tokens/auth headers.

## Documentation

Detailed material was split out of this README:

- [`docs/01-project-contract.md`](docs/01-project-contract.md)
- [`docs/02-provider-capabilities.md`](docs/02-provider-capabilities.md)
- [`docs/03-system-architecture.md`](docs/03-system-architecture.md)
- [`docs/04-evidence-extraction-replay.md`](docs/04-evidence-extraction-replay.md)
- [`docs/05-learning-loop.md`](docs/05-learning-loop.md)
- [`docs/06-operations-roadmap.md`](docs/06-operations-roadmap.md)
- [`SCORE.md`](SCORE.md)

## Current Status

```text
status = architecture_defined ∧ implementation_unproven
```

The next implementation target is the minimal provider registry plus `chatgpt_private` send/read loop.
