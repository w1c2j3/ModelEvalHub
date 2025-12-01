export interface ModelFamily {
  id: string;
  project_id: string;
  name: string;
  model_type: string;
  description?: string | null;
  created_at: string;
  updated_at: string;
}

export interface Run {
  id: string;
  experiment_id: string;
  project_id: string;
  model_impl_id: string;
  checkpoint_id: string;
  task_id: string;
  run_type: string;
  status: string;
  created_at?: string;
  started_at?: string | null;
  finished_at?: string | null;
}

