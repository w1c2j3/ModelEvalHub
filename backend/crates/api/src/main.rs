use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use deadpool_redis::{Config as RedisConfig, Pool as RedisPool, Runtime};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use unified_domain::datasets::{self, Dataset, NewDataset};
use unified_domain::experiments::{self, Experiment, NewExperiment};
use unified_domain::metrics;
use unified_domain::models::{
    self, Checkpoint, ModelFamily, ModelImplementation, NewCheckpoint, NewModelFamily,
    NewModelImplementation,
};
use unified_domain::projects::{self, NewProject, Project};
use unified_domain::runs::{self, NewRun, Run};
use unified_domain::sample_outputs;
use unified_domain::tasks::{self, NewTask, Task};
use unified_shared::error::DomainError;
use unified_shared::eval::{EvalConfig, RunStatus};
use unified_shared::settings::Settings;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: unified_domain::db::DbPool,
    redis: RedisPool,
    settings: Settings,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let settings = Settings::load()?;
    let db = unified_domain::db::init_pool(&settings.database.url).await?;
    let redis_cfg = RedisConfig::from_url(settings.redis.url.clone());
    let redis = redis_cfg.create_pool(Some(Runtime::Tokio1))?;

    let state = AppState {
        db,
        redis,
        settings: settings.clone(),
    };

    let app = Router::new()
        .route("/healthz", get(health_check))
        .route("/projects", get(list_projects).post(create_project))
        .nest(
            "/models",
            Router::new()
                .route(
                    "/families",
                    get(list_model_families).post(create_model_family),
                )
                .route("/impls", get(list_model_impls).post(create_model_impl))
                .route(
                    "/checkpoints",
                    get(list_checkpoints).post(create_checkpoint),
                ),
        )
        .route("/datasets", get(list_datasets).post(create_dataset))
        .route("/tasks", get(list_tasks).post(create_task))
        .route(
            "/experiments",
            get(list_experiments).post(create_experiment),
        )
        .route("/experiments/:id/compile", post(compile_experiment))
        .route("/runs", get(list_runs))
        .route("/runs/:id", get(get_run))
        .route("/runs/:id/enqueue", post(enqueue_run))
        .route("/metrics", get(list_metrics))
        .route("/samples", get(list_samples))
        .route("/tests/trigger", post(trigger_remote_test))
        .with_state(Arc::new(state));

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn health_check() -> &'static str {
    "ok"
}

fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry().with(fmt_layer).init();
}

type SharedState = Arc<AppState>;

#[derive(Deserialize)]
struct ProjectQuery {
    project_id: Uuid,
}

#[derive(Deserialize)]
struct RunQuery {
    run_id: Uuid,
}

async fn list_projects(
    State(state): State<SharedState>,
) -> Result<Json<Vec<Project>>, DomainError> {
    let projects = projects::list(&state.db).await?;
    Ok(Json(projects))
}

#[derive(Deserialize)]
struct CreateProjectRequest {
    name: String,
    description: Option<String>,
}

async fn create_project(
    State(state): State<SharedState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<Project>, DomainError> {
    let project = projects::create(
        &state.db,
        NewProject {
            name: payload.name,
            description: payload.description,
        },
    )
    .await?;
    Ok(Json(project))
}

async fn list_model_families(
    State(state): State<SharedState>,
    Query(query): Query<ProjectQuery>,
) -> Result<Json<Vec<ModelFamily>>, DomainError> {
    let items = models::list_families(&state.db, &query.project_id).await?;
    Ok(Json(items))
}

#[derive(Deserialize)]
struct CreateModelFamilyRequest {
    project_id: Uuid,
    name: String,
    model_type: String,
    description: Option<String>,
}

async fn create_model_family(
    State(state): State<SharedState>,
    Json(payload): Json<CreateModelFamilyRequest>,
) -> Result<Json<ModelFamily>, DomainError> {
    let item = models::create_family(
        &state.db,
        NewModelFamily {
            project_id: payload.project_id,
            name: payload.name,
            model_type: payload.model_type,
            description: payload.description,
        },
    )
    .await?;
    Ok(Json(item))
}

async fn list_model_impls(
    State(state): State<SharedState>,
    Query(query): Query<ProjectQuery>,
) -> Result<Json<Vec<ModelImplementation>>, DomainError> {
    let items = models::list_impls(&state.db, &query.project_id).await?;
    Ok(Json(items))
}

#[derive(Deserialize)]
struct CreateModelImplRequest {
    project_id: Uuid,
    family_id: Uuid,
    name: String,
    repo_url: Option<String>,
    repo_reference: Option<String>,
    runtime_type: String,
    config_path: Option<String>,
    default_task_types: Vec<String>,
}

async fn create_model_impl(
    State(state): State<SharedState>,
    Json(payload): Json<CreateModelImplRequest>,
) -> Result<Json<ModelImplementation>, DomainError> {
    let item = models::create_impl(
        &state.db,
        NewModelImplementation {
            project_id: payload.project_id,
            family_id: payload.family_id,
            name: payload.name,
            repo_url: payload.repo_url,
            repo_reference: payload.repo_reference,
            runtime_type: payload.runtime_type,
            config_path: payload.config_path,
            default_task_types: payload.default_task_types,
        },
    )
    .await?;
    Ok(Json(item))
}

#[derive(Deserialize)]
struct ListCheckpointsQuery {
    model_impl_id: Uuid,
}

async fn list_checkpoints(
    State(state): State<SharedState>,
    Query(query): Query<ListCheckpointsQuery>,
) -> Result<Json<Vec<Checkpoint>>, DomainError> {
    let items = models::list_checkpoints(&state.db, &query.model_impl_id).await?;
    Ok(Json(items))
}

#[derive(Deserialize)]
struct CreateCheckpointRequest {
    project_id: Uuid,
    model_impl_id: Uuid,
    name: String,
    weights_uri: Option<String>,
    step: Option<i64>,
    training_summary: Option<Value>,
}

async fn create_checkpoint(
    State(state): State<SharedState>,
    Json(payload): Json<CreateCheckpointRequest>,
) -> Result<Json<Checkpoint>, DomainError> {
    let item = models::create_checkpoint(
        &state.db,
        NewCheckpoint {
            project_id: payload.project_id,
            model_impl_id: payload.model_impl_id,
            name: payload.name,
            weights_uri: payload.weights_uri,
            step: payload.step,
            training_summary: payload.training_summary,
        },
    )
    .await?;
    Ok(Json(item))
}

async fn list_datasets(
    State(state): State<SharedState>,
    Query(query): Query<ProjectQuery>,
) -> Result<Json<Vec<Dataset>>, DomainError> {
    let items = datasets::list(&state.db, &query.project_id).await?;
    Ok(Json(items))
}

#[derive(Deserialize)]
struct CreateDatasetRequest {
    project_id: Uuid,
    name: String,
    version: Option<String>,
    storage_uri: Option<String>,
    schema: Option<Value>,
    num_samples: Option<i64>,
}

async fn create_dataset(
    State(state): State<SharedState>,
    Json(payload): Json<CreateDatasetRequest>,
) -> Result<Json<Dataset>, DomainError> {
    let item = datasets::create(
        &state.db,
        NewDataset {
            project_id: payload.project_id,
            name: payload.name,
            version: payload.version,
            storage_uri: payload.storage_uri,
            schema: payload.schema,
            num_samples: payload.num_samples,
        },
    )
    .await?;
    Ok(Json(item))
}

async fn list_tasks(
    State(state): State<SharedState>,
    Query(query): Query<ProjectQuery>,
) -> Result<Json<Vec<Task>>, DomainError> {
    let items = tasks::list(&state.db, &query.project_id).await?;
    Ok(Json(items))
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    project_id: Uuid,
    dataset_id: Uuid,
    name: String,
    task_type: String,
    eval_engine: String,
    eval_config: Value,
    default_metrics: Option<Value>,
}

async fn create_task(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<Task>, DomainError> {
    let task = tasks::create(
        &state.db,
        NewTask {
            project_id: payload.project_id,
            dataset_id: payload.dataset_id,
            name: payload.name,
            task_type: payload.task_type,
            eval_engine: payload.eval_engine,
            eval_config: payload.eval_config,
            default_metrics: payload.default_metrics,
        },
    )
    .await?;
    Ok(Json(task))
}

async fn list_experiments(
    State(state): State<SharedState>,
    Query(query): Query<ProjectQuery>,
) -> Result<Json<Vec<Experiment>>, DomainError> {
    let items = experiments::list(&state.db, &query.project_id).await?;
    Ok(Json(items))
}

#[derive(Deserialize)]
struct CreateExperimentRequest {
    project_id: Uuid,
    name: String,
    description: Option<String>,
    scenario_type: Option<String>,
    tasks: Vec<Uuid>,
    global_config: Option<Value>,
}

async fn create_experiment(
    State(state): State<SharedState>,
    Json(payload): Json<CreateExperimentRequest>,
) -> Result<Json<Experiment>, DomainError> {
    let experiment = experiments::create(
        &state.db,
        NewExperiment {
            project_id: payload.project_id,
            name: payload.name,
            description: payload.description,
            scenario_type: payload.scenario_type,
            tasks: payload.tasks,
            global_config: payload.global_config,
        },
    )
    .await?;
    Ok(Json(experiment))
}

#[derive(Deserialize)]
struct CompileExperimentRequest {
    runs: Vec<CompileRunRequest>,
}

#[derive(Deserialize)]
struct CompileRunRequest {
    model_impl_id: Uuid,
    checkpoint_id: Uuid,
    task_id: Uuid,
    run_type: Option<String>,
    eval_config: Value,
}

#[derive(Serialize)]
struct CompileExperimentResponse {
    run_ids: Vec<Uuid>,
}

async fn compile_experiment(
    State(state): State<SharedState>,
    Path(experiment_id): Path<Uuid>,
    Json(payload): Json<CompileExperimentRequest>,
) -> Result<Json<CompileExperimentResponse>, DomainError> {
    let experiment = experiments::get(&state.db, &experiment_id).await?;
    let mut created = Vec::new();
    for run_req in payload.runs {
        let mut config = run_req.eval_config;
        if let Some(obj) = config.as_object_mut() {
            obj.insert(
                "experiment_id".into(),
                Value::String(experiment_id.to_string()),
            );
            obj.insert(
                "project_id".into(),
                Value::String(experiment.project_id.to_string()),
            );
        }
        let new_run = NewRun {
            experiment_id,
            project_id: experiment.project_id,
            model_impl_id: run_req.model_impl_id,
            checkpoint_id: run_req.checkpoint_id,
            task_id: run_req.task_id,
            run_type: run_req.run_type.unwrap_or_else(|| "offline_eval".into()),
            status: RunStatus::Queued,
            eval_config: config,
        };
        let run = runs::create(&state.db, new_run).await?;
        created.push(run.id);
    }

    Ok(Json(CompileExperimentResponse { run_ids: created }))
}

async fn list_runs(
    State(state): State<SharedState>,
    Query(query): Query<ProjectQuery>,
) -> Result<Json<Vec<Run>>, DomainError> {
    let items = runs::list(&state.db, &query.project_id).await?;
    Ok(Json(items))
}

async fn get_run(
    State(state): State<SharedState>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<Run>, DomainError> {
    let run = runs::get(&state.db, &run_id).await?;
    Ok(Json(run))
}

#[derive(Serialize)]
struct EnqueueResponse {
    accepted: bool,
}

async fn enqueue_run(
    State(state): State<SharedState>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<EnqueueResponse>, DomainError> {
    let run = runs::get(&state.db, &run_id).await?;
    let payload = serde_json::to_string(&run.eval_config)
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let mut redis_conn = state
        .redis
        .get()
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    redis_conn
        .rpush(&state.settings.redis.queue_key, payload)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(Json(EnqueueResponse { accepted: true }))
}

async fn list_metrics(
    State(state): State<SharedState>,
    Query(query): Query<RunQuery>,
) -> Result<Json<Vec<metrics::Metric>>, DomainError> {
    let items = metrics::list_by_run(&state.db, &query.run_id).await?;
    Ok(Json(items))
}

async fn list_samples(
    State(state): State<SharedState>,
    Query(query): Query<RunQuery>,
) -> Result<Json<Vec<sample_outputs::SampleOutput>>, DomainError> {
    let items = sample_outputs::list_by_run(&state.db, &query.run_id).await?;
    Ok(Json(items))
}

#[derive(Deserialize)]
struct RemoteTestRequest {
    project_id: Uuid,
    description: Option<String>,
}

#[derive(Serialize)]
struct RemoteTestResponse {
    message: String,
}

async fn trigger_remote_test(Json(payload): Json<RemoteTestRequest>) -> impl IntoResponse {
    let body = RemoteTestResponse {
        message: format!(
            "Remote test placeholder queued for project {}",
            payload.project_id
        ),
    };
    (StatusCode::ACCEPTED, Json(body))
}
