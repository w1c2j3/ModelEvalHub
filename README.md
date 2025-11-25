# ModelEvalHub

A unified platform for evaluating AI models, integrating features from MLFlow, OpenCompass, Weights & Biases, and Benchmark Dashboards. This platform provides tools for model management, experiment tracking, and comprehensive evaluation across multiple AI tasks such as LLM, image generation, and more.

## Features

- **Model Registration**: Register models, track weights, and manage multiple versions of models.
- **Experiment Tracking**: Easily create, manage, and track experiments with detailed results comparison.
- **Comprehensive Metrics**: Track scalar, distribution, time-series metrics, and multi-modal evaluation.
- **Task and Dataset Management**: Organize tasks, datasets, and benchmark tasks for efficient evaluation.
- **Frontend UI**: Interactive and beautiful dashboard to visualize model performance and metrics.

## Architecture Overview

The system is composed of four main components:

1. **Frontend (Web UI)**:
   - Built with React and Ant Design, provides an interactive user interface for model registration, experiment management, and result visualization.
   
2. **API Backend**:
   - Powered by FastAPI to manage model registry, experiments, task submission, and resource scheduling.
   
3. **Execution Layer**:
   - Python-based workers to execute training, inference, and evaluation tasks, with multi-GPU and distributed system support.

4. **Storage Layer**:
   - PostgreSQL for metadata storage, MinIO/S3 for model weights and outputs, and Prometheus for system monitoring.

## Installation

### Prerequisites

- Python 3.8+
- Docker (for containerized environments)
- PostgreSQL / MinIO / S3

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/ModelEvalHub.git
   cd ModelEvalHub
