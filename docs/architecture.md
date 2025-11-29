# Architecture Overview

- **Backend**: Rust workspace with crates for API, domain/services, worker, integrations, shared types.
- **Frontend**: Vue 3 + TypeScript + Vite.
- **Queue**: Redis (RQ-style semantics) for run dispatch.
- **Eval Engines**: Integrations call external frameworks (lm-eval-harness etc.) via subprocess.
- **Result Flow**: `EvalConfig` → Worker → Python runner → `EvalResult` → `ResultStore` (DB today, ClickHouse/S3 later).

