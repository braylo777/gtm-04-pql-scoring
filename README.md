# gtm-04-pql-scoring

**GTM Engineering — Stage 4 of 12: PLG Telemetry & PQL Scoring**

Find the free accounts worth a sales call: product events to account features, explainable PQL scores (fit × usage), the 'aha moment' threshold that predicts retention, and feature adoption curves.

**Live demo:** https://gtm-04-pql-scoring.braylo777.workers.dev — the Rust core runs in your browser as WebAssembly. No server, no tracking.

## Why this matters

In product-led growth, the product does the first sales call. Most free users never pay, and sales cannot call them all. A Product Qualified Lead score tells sales which accounts already got value and fit the ideal customer profile. It only works if each point is explainable, the 'aha' threshold comes from real retention data, and the telemetry is clean (no duplicate retries, no orphan events).

## What it does

- Segment / RudderStack-style track stream to per-account features: event counts, distinct users, active days; duplicate messageId and orphan events counted
- PQL score: capped points per feature, firmographic fit multipliers (range or set rules), threshold, next action, one reason per point
- Aha-moment search: every threshold, retention above vs below, minimum group size, best lift, with a correlation-not-causation caution
- Adoption curve: cumulative adoption % per week, day to 50%, median first-use day, features ranked

## What it does not do (yet)

- No live Segment / RudderStack / warehouse connection: events are synthetic JSON
- Aha search is univariate (one action count); no multivariate model or significance test
- Weights in the PQL model are set by hand; no model training from closed-won data

## Operations

Every operation takes JSON in and gives JSON out. The CLI, the tests and the demo page all use the same entry point.

| Operation | What it does |
|---|---|
| `track_to_features` | Roll a track-event stream up to account features. One retried duplicate and one orphan event are in the sample. |
| `pql_score` | Score each account: usage points (capped per feature) × fit multipliers. Accounts above the threshold alert an AE. |
| `aha_threshold` | Find the first-week action count that best separates retained users from churned users (40 synthetic users). |
| `adoption_curve` | Cumulative feature adoption per week since launch for 20 eligible accounts. |

## Run it

You need Rust (stable). No Node, no npm.

```bash
git clone https://github.com/braylo777/gtm-04-pql-scoring
cd gtm-04-pql-scoring
cargo test
cargo run -- ops
```

Try the first demo example from the command line:

```bash
python3 -c 'import json;print(json.dumps(json.load(open("web/examples.json"))["ops"][0]["input"]))' | cargo run -q -- track_to_features
```

## Build and deploy the demo

```bash
./build.sh    # tests, then compiles the core to web/app.wasm
./deploy.sh   # serves web/ as a Cloudflare Workers static-assets site (needs a Cloudflare login)
```

`web/` is a plain static site (`index.html`, `app.js`, `app.wasm`). It has no build step, so any static host works.

## Layout

```
src/lib.rs        operation list and router
src/api.rs        JSON entry point + the 3 WASM exports (alloc, dealloc, run)
src/main.rs       CLI
src/*.rs          one module per capability, each with unit tests
tests/            integration tests (every demo example must pass)
web/              demo page; examples.json drives both the page and this README
```

## Design choices

- **Rust, few dependencies.** Only `serde_json`, pinned to an exact version with a committed `Cargo.lock`.
- **Deterministic.** Time, randomness and keys are inputs, so every result can be replayed in a test or an audit.
- **Fail closed.** Unknown input or missing fields give an error, never a silent default.

## Part of a series

This is one of 12 repos that work through a public Go-To-Market (GTM) Engineering roadmap: [12 stages by Suraj Sharma (@suraj_sharma14)](https://x.com/suraj_sharma14/status/2125552583452270692). The roadmap is the inspiration; the code here is original. All 12: [github.com/braylo777?q=gtm-](https://github.com/braylo777?tab=repositories&q=gtm-).

## Data and privacy

Every example uses synthetic data. Nothing calls a paid API, scrapes a site, identifies a real visitor or sends an email.

## License

MIT — see [LICENSE](LICENSE). Built by [Braylo SI](https://braylo.si).
