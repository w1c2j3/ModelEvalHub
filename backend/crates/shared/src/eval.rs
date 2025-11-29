use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub type Timestamp = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvalEngine {
    LmEvalHarness,
    OpenCompass,
    Helm,
    DeepEval,
    OpenAiEvals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Qa,
    Summarization,
    Rag,
    CodeGen,
    Classification,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalConfig {
    pub run_id: Uuid,
    pub project_id: Uuid,
    pub engine: EvalEngine,
    pub engine_version: Option<String>,
    pub model: ModelConfig,
    pub dataset: DatasetConfig,
    pub task: TaskConfig,
    pub metrics: Vec<MetricConfig>,
    pub sampling: SamplingConfig,
    pub resources: ResourceConfig,
    pub output: OutputConfig,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub logical_name: String,
    pub provider: String,
    pub model_name: String,
    pub endpoint: Option<String>,
    pub api_key_ref: Option<String>,
    pub extra: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetConfig {
    pub source: DatasetSource,
    pub name: String,
    pub split: Option<String>,
    pub uri: Option<String>,
    pub filters: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DatasetSource {
    BuiltIn,
    Uploaded,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub task_type: TaskType,
    pub task_name: String,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricConfig {
    pub name: String,
    pub metric_type: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub priority: Option<u8>,
    pub num_gpus: Option<u8>,
    pub gpu_type: Option<String>,
    pub cpu_cores: Option<u8>,
    pub memory_gb: Option<u16>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum OutputConfig {
    DbOnly,
    ObjectStore { samples_uri: String, format: String },
    ClickHouse { table: String },
    Hybrid { ch_table: String, samples_uri: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub run_id: Uuid,
    pub status: RunStatus,
    pub started_at: Timestamp,
    pub completed_at: Timestamp,
    pub metrics: Vec<MetricRecord>,
    pub samples: SampleResultLocation,
    pub error: Option<EvalErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRecord {
    pub run_id: Uuid,
    pub dataset: String,
    pub subset: Option<String>,
    pub split: Option<String>,
    pub metric_name: String,
    pub value: f64,
    pub n_samples: Option<i64>,
    pub ci_low: Option<f64>,
    pub ci_high: Option<f64>,
    pub extra: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleRecord {
    pub run_id: Uuid,
    pub dataset: String,
    pub subset: Option<String>,
    pub split: Option<String>,
    pub sample_index: i64,
    pub input: String,
    pub reference: Option<String>,
    pub output: String,
    pub metrics: Option<Value>,
    pub latency_ms: Option<i64>,
    pub token_counts: Option<TokenCount>,
    pub error: Option<SampleError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCount {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum SampleResultLocation {
    Inline { samples: Vec<SampleRecord> },
    ObjectStore { uri: String, format: String },
    ClickHouse { table: String },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleError {
    pub message: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalErrorPayload {
    pub kind: EvalErrorKind,
    pub message: String,
    pub code: Option<String>,
    pub engine: Option<String>,
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalErrorKind {
    Config,
    Engine,
    Infra,
    Timeout,
    Cancelled,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RunStatus {
    Queued,
    Running,
    Completed,
    FailedConfig,
    FailedEngine,
    FailedInfra,
    TimedOut,
    Cancelled,
}

