import axios from "axios";

export function useHealthApi() {
  const client = axios.create({
    baseURL: "/api",
  });

  return {
    async check(): Promise<string> {
      const res = await client.get("/healthz");
      return res.data ?? "ok";
    },
  };
}

