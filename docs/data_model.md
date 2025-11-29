# Data Model

| Entity        | Key fields                                                                 |
|---------------|-----------------------------------------------------------------------------|
| `projects`    | `id`, `name`, `created_at`                                                  |
| `models`      | `id`, `project_id`, `provider`, `config_json`                               |
| `datasets`    | `id`, `project_id`, `source`, `schema_json`                                 |
| `tasks`       | `id`, `project_id`, `task_type`, `engine`, `eval_config_json`               |
| `experiments` | `id`, `project_id`, `scenario_type`, `global_config_json`                   |
| `runs`        | `id`, `experiment_id`, `project_id`, `status`, `eval_config_json`, `error`  |
| `metrics`     | `id`, `run_id`, `dataset`, `metric_name`, `value`, `extra_json`             |
| `sample_outputs` | `run_id`, `dataset`, `sample_index`, `payload_json`, `storage_uri`       |

See `.cursor/rules/07-eval-domain.mdc` for JSON schema definitions shared across services.

