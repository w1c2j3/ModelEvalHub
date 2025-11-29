# API Design (Phase 1)

| Route                         | Method | Description                              |
|------------------------------|--------|------------------------------------------|
| `/healthz`                   | GET    | Liveness probe                           |
| `/models`                    | CRUD   | Manage model families & implementations  |
| `/datasets`                  | CRUD   | Register datasets                         |
| `/tasks`                     | CRUD   | Define evaluation tasks                   |
| `/experiments`               | GET/POST | Create + list experiments                |
| `/experiments/{id}/compile`  | POST   | Generate runs from an experiment         |
| `/runs`                      | GET    | List/filter runs                          |
| `/runs/{id}`                 | GET    | Run details + metrics/samples summary     |
| `/runs/{id}/enqueue`         | POST   | Enqueue run to Redis queue                |
| `/metrics?run_id=...`        | GET    | Fetch metrics for a run                   |
| `/samples?run_id=...`        | GET    | Fetch sample outputs                      |
| `/tests/trigger`             | POST   | Remote test placeholder (future feature)  |

