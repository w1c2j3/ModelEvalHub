import { http } from "./http";
import type { ModelFamily } from "@/types";

export async function listModelFamilies(projectId: string): Promise<ModelFamily[]> {
  const res = await http.get<ModelFamily[]>("/models/families", {
    params: { project_id: projectId },
  });
  return res.data;
}

