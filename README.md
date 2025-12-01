# ModelEvalHub

# Unified AI Evaluation Platform
---

## Motivation

Modern AI systems span **multiple model families** (LLMs, diffusion, multi-modal, etc.),  
with **multiple weights and runtimes** (HF, vLLM, custom trainers), evaluated on  
**multiple benchmarks** (accuracy, speed, resource usage, human/LLM-as-judge, …).

Existing tools (MLflow, W&B, lm-eval-harness, OpenCompass, HELM, HEIM, Langfuse, Opik, …)  
cover parts of this stack, but there is no **single, unified platform** that:

- treats **model + checkpoint + runtime + task** as first-class objects,
- runs **comparable experiments** across families and modalities,
- collects **training + inference + generation quality** metrics in one place,
- provides a **beautiful, product-level UI** for inspection and comparison.

This project aims to fill that gap.

---

## High-level Goals

- **Unified model registry**  
  - Register model families (LLM / T2I / VLM / …), implementations (code repo + runtime), and checkpoints (weights + training metadata).
  - Store **only metadata**; actual code and weights can live on Git, HF Hub, S3, local FS, etc.

- **Task & benchmark management**  
  - Define tasks (QA, reasoning, code generation, translation, T2I, …) and bind them to datasets.
  - Compose tasks into benchmarks and scenarios with task-specific metric suites.

- **Experiment & run orchestration**  
  - Express experiments like:
    - same model, different checkpoints (training progress comparison),
    - same checkpoint, different runtimes (HF vs vLLM vs custom runtime),
    - different model families on the same benchmark.
  - Orchestrate runs via a queue-based worker system (single machine first, cluster-ready later).

- **Metrics & artifacts store**  
  - Store scalar metrics (loss, accuracy, latency, throughput, mem usage, …),
  - timeseries (training curves),
  - sample-level artifacts (prompts, generations, reference outputs, images, videos, logs).

- **Modern UI**  
  - Dashboards for experiments, runs, metrics, curves, and sample-level comparison,
  - model & task management pages,
  - LLM-style text comparison view, T2I-style image grid (later phases).

---

## Architecture Overview

Conceptually, the platform is split into the following layers:

- **Model Registry**
  - `ModelFamily` – LLM / Diffusion / VLM / …
  - `ModelImpl` – code repo + runtime type (HF, vLLM, HTTP API, custom trainer, …)
  - `Checkpoint` – weights + metadata (step, epoch, dataset version, training config)
  - In later phases: runtime variants to compare the same weights under different backends.

- **Task & Benchmark Registry**
  - `Dataset` – name, version, storage location, schema.
  - `Task` – task type (QA, MT, code, T2I, …) + dataset + input/output schema.
  - `Benchmark` / `Scenario` – compositions of tasks with:
    - metric suites (BLEU, ROUGE, FID, CLIPScore, latency, throughput, …),
    - run configuration (batch size, hardware constraints, sampling params).

- **Experiment & Run**
  - `Experiment` – logical experiment definition:
    - scenario,
    - selected models / checkpoints / runtimes,
    - global config (seed, sample limits, resource constraints).
  - `Run` – atomic execution unit:
    - one (model impl + checkpoint + runtime) on one task,
    - run type: `training`, `inference`, or `offline_eval`,
    - status, metrics, logs, artifacts.

- **Execution Layer (Orchestrator + Workers)**
  - **Orchestrator**
    - Takes Experiment definitions and compiles them into concrete `Run` specs.
    - Submits runs into a queue (local or Redis-based for Phase 1).
  - **Workers**
    - Pull Run specs, prepare the model via a **Model Adapter**,
    - execute training / inference / evaluation,
    - emit a stream of `RunEvent` objects (metrics, samples, logs, resource snapshots).

- **Plugin System**
  - **Model Adapters**
    - Standardized interface for:
      - training: `prepare_train`, `train_step`, `eval_step`,
      - inference: `prepare_infer`, `infer`.
    - Different adapters for LLMs, T2I, speech, etc., all presenting the same surface to workers.
  - **Eval Engines**
    - Bridge external evaluation frameworks into this platform (lm-eval-harness, OpenCompass, HELM, HEIM, T2I-Eval, …).
  - **Metric Plugins**
    - Subscribe to `RunEvent`s (train_step, eval_step, sample_output, resource_usage, …),
    - compute metrics like BLEU, ROUGE, grad_norm histograms, FID, CLIPScore, aesthetic scores, latency p95, etc.

- **Frontend**
  - Modern React-based dashboard with:
    - Model & checkpoint browser,
    - Task / benchmark / scenario management,
    - Experiment & run list and detail pages,
    - Curve plots, metric tables, and sample-level comparison views.

---

## Phase 1 Scope (MVP)

Phase 1 focuses on **LLM evaluation + basic training monitoring**:

- **Supported model type**
  - Text-only LLMs (instruction-following, QA, reasoning, code-gen).

- **Model registration**
  - A constrained **model plugin** mechanism:
    - code repo with a `ai_eval_config.yaml`,
    - a simple LLM adapter with a fixed `generate(prompts, params)` interface,
    - weights stored locally or in a centralized storage (platform downloads them).

- **Tasks & benchmarks**
  - A small set of core LLM tasks:
    - e.g. MMLU, GSM8K, simple QA / code tasks.
  - Integrated via **one main eval engine**:
    - e.g. `lm-evaluation-harness` or a subset of OpenCompass.

- **Experiments & runs**
  - Compare:
    - same model, different checkpoints,
    - different models on the same tasks.
  - Single-machine, multi-GPU workers with basic resource limits.

- **Metrics**
  - Accuracy / exact match / task-specific scores,
  - latency and throughput,
  - training curves (loss, grad_norm, samples/sec) via a simple logging API.

- **UI**
  - Dashboard for recent experiments & active runs,
  - Experiment detail with:
    - configuration summary,
    - run-level metric table,
    - curves (loss, accuracy, latency).
  - Sample-level LLM output comparison (prompt + multiple model answers + scores).

Later phases will extend this to **text-to-image, multi-modal, advanced error slicing, and human/LLM-as-a-judge evaluations**.

---

## Technology Stack

- **Backend**
  - Rust workspace (Axum for HTTP, SQLx/SeaORM-ready domain layer)
  - Redis (deadpool-redis) for run queues
  - Worker crate executes subprocess integrations (lm-eval-harness, OpenCompass, HELM, etc.)

- **Frontend**
  - Vue 3 + TypeScript + Vite
  - Basic Pinia store + Vue Router for navigation

- **Storage**
  - MySQL for metadata tables (projects, models, datasets, runs, metrics)
  - ClickHouse / Object storage planned via `OutputConfig` once scale requires it

This stack keeps the orchestrator strongly typed end-to-end while remaining compatible with Python-based eval frameworks via subprocess adapters.

## Getting Started

1. **Backend**
   ```bash
   cd backend
   cargo fmt
   cargo check
   cargo run -p unified-api
   ```
   The server expects a MySQL instance (see `backend/config/default.toml`). Update that file or set `UEP__DATABASE__URL` env var, then run migrations manually (schema definition is documented in `.cursor/rules/07-eval-domain.mdc`).

2. **Worker**
   ```bash
   cd backend
   cargo run -p unified-worker
   ```
   Workers pull runs from Redis and transition them through `queued -> running -> completed` (the execution logic is stubbed for Phase 1 but already updates DB state).

3. **Frontend**
   ```bash
   cd frontend
   npm install
   npm run dev
   ```
   The Vue app proxies `/api/*` to the Rust backend. Use the Models/Runs pages to query data by `project_id`.

---

## Related & Referenced Projects

This project draws heavy inspiration from and may integrate with:

- **LLM evaluation frameworks**
  - [EleutherAI / lm-evaluation-harness](https://github.com/EleutherAI/lm-evaluation-harness)
  - [OpenCompass](https://github.com/open-compass/opencompass)
  - [Stanford HELM](https://github.com/stanford-crfm/helm)
  - [OpenAI Evals](https://github.com/openai/evals)
  - [OpenAI Simple Evals](https://github.com/openai/simple-evals)
  - [DeepEval](https://github.com/confident-ai/deepeval)

- **Text-to-Image / multi-modal evaluation**
  - HEIM (within the HELM repo) – holistic text-to-image evaluation
  - [text2image-benchmark](https://github.com/boomb0om/text2image-benchmark)
  - [Benchmarking Awesome Diffusion Models](https://github.com/Schuture/Benchmarking-Awesome-Diffusion-Models)
  - [T2I-Eval](https://github.com/maziao/T2I-Eval)
  - [CLIPScore](https://github.com/jmhessel/clipscore)
  - [encord text-to-image-eval](https://github.com/encord-team/text-to-image-eval)

- **LLM application observability & evaluation platforms**
  - [Langfuse](https://github.com/langfuse/langfuse)
  - [Opik (by Comet)](https://github.com/comet-ml/opik)

In development setups, these projects may be cloned under a dedicated directory (e.g. `third_party/`) for easier experimentation and integration.

---

## Roadmap (Draft)

- **Phase 1 – LLM MVP**
  - Core domain model: Model / Checkpoint / Task / Experiment / Run / Metric / SampleOutput
  - Model plugin spec for LLMs + local HF runtime
  - Integration with one main eval engine (e.g. lm-eval-harness)
  - Basic dashboard, experiment view, and LLM output comparison

- **Phase 2 – Text-to-Image & Multi-modal**
  - T2I tasks & datasets
  - Integration with HEIM / text2image-benchmark / CLIPScore
  - Image grid views and sample-level visualization

- **Phase 3 – Advanced Analysis & Automation**
  - Error slicing and subset analysis
  - Periodic / scheduled evaluations
  - Human & LLM-as-a-judge flows
  - Deeper integration with tracing/observability (Langfuse/Opik-like features)

---

## Status

This project is currently under active design and early implementation.  
Breaking changes are expected. Contributions and feedback are welcome once the core MVP is stable.
