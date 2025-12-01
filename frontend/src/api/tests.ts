import { http } from "./http";

interface RemoteTestPayload {
  project_id: string;
  description?: string;
}

export async function triggerRemoteTest(payload: RemoteTestPayload) {
  await http.post("/tests/trigger", payload);
}

