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

Its current architecture is evidence-driven. Its target architecture is data-driven.

```text
request
  → provider selection
  → capability plan
  → browser action
  → evidence capture
  → dataset registry
  → feature extraction
  → pattern mining
  → policy decision
  → response extraction
  → feedback + replay/privacy verification
  → policy update
  → OpenAI-compatible response
```

## Core Equation

```text
evidence_driven = evidence → verify(action)

data_driven = data → mining → policy → action → feedback → update(policy)

Good = max(evidence_integrity, data_discovery, policy_learning, replay_determinism, capability_adaptation, privacy_safety)
```

One-line explanation: the project becomes good when browser evidence becomes reusable data, mined patterns update policy, and policy changes improve future provider behavior.

## Data Discovery

Data discovery means the router turns browser evidence into reusable datasets and mines them for stable provider structure.

```text
discover = evidence → dataset → features → patterns → hypotheses
```

Discovery finds UI targets, stream shapes, upload states, group-chat authorship signals, terminality signals, and failure patterns.

Discovery does not mine private prompt content, assistant content, credentials, cookies, or unrelated human messages.

## Learning

Learning means the router converts verified discovery into policy updates that change future behavior.

```text
learning = hypotheses → score → update policy → act → measure feedback
```

Learning may use raw evidence in controlled discovery mode. Long-term promotion uses schemas, action receipts, discovery records, and replay-passing rules under retention policy.

## Scope

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

- [`docs/01-project-contract.md`](docs/01-project-contract.md)
- [`docs/02-provider-capabilities.md`](docs/02-provider-capabilities.md)
- [`docs/03-system-architecture.md`](docs/03-system-architecture.md)
- [`docs/04-evidence-extraction-replay.md`](docs/04-evidence-extraction-replay.md)
- [`docs/05-data-discovery.md`](docs/05-data-discovery.md)
- [`docs/06-learning-loop.md`](docs/06-learning-loop.md)
- [`docs/07-operations-roadmap.md`](docs/07-operations-roadmap.md)
- [`SCORE.md`](SCORE.md)

## Current Status

```text
status = architecture_defined ∧ implementation_unproven
```

The next implementation target is the minimal provider registry plus `chatgpt_private` send/read loop.
