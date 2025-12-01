import { http } from "./http";
import type { Run } from "@/types";

export async function listRuns(projectId: string): Promise<Run[]> {
  const res = await http.get<Run[]>("/runs", {
    params: { project_id: projectId },
  });
  return res.data;
}

