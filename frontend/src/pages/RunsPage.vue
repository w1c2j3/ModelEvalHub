<script setup lang="ts">
import { ref, onMounted } from "vue";
import type { Run } from "@/types";
import { listRuns } from "@/api/runs";

const projectId = ref("");
const runs = ref<Run[]>([]);
const loading = ref(false);
const errorMessage = ref<string | null>(null);

async function load() {
  if (!projectId.value) {
    runs.value = [];
    return;
  }
  loading.value = true;
  errorMessage.value = null;
  try {
    runs.value = await listRuns(projectId.value);
  } catch (err) {
    errorMessage.value = (err as Error).message;
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  const stored = localStorage.getItem("project_id");
  if (stored) {
    projectId.value = stored;
    load();
  }
});

function handleSubmit() {
  localStorage.setItem("project_id", projectId.value);
  load();
}
</script>

<template>
  <section>
    <h2>Runs</h2>
    <form @submit.prevent="handleSubmit" class="project-form">
      <label>
        Project ID
        <input v-model="projectId" placeholder="UUID" />
      </label>
      <button type="submit">Load</button>
    </form>

    <p v-if="loading">Loading...</p>
    <p v-else-if="errorMessage" class="error">{{ errorMessage }}</p>

    <table v-else>
      <thead>
        <tr>
          <th>Run ID</th>
          <th>Run Type</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="run in runs" :key="run.id">
          <td>{{ run.id }}</td>
          <td>{{ run.run_type }}</td>
          <td>{{ run.status }}</td>
        </tr>
        <tr v-if="runs.length === 0">
          <td colspan="3">No runs yet.</td>
        </tr>
      </tbody>
    </table>
  </section>
</template>

<style scoped>
.project-form {
  display: flex;
  gap: 1rem;
  margin-bottom: 1rem;
}

input {
  padding: 0.4rem 0.6rem;
  border-radius: 6px;
  border: 1px solid #d0d7e3;
  min-width: 280px;
}

button {
  padding: 0.4rem 1rem;
  border: none;
  border-radius: 6px;
  background: #2563eb;
  color: white;
  cursor: pointer;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  padding: 0.5rem;
  border-bottom: 1px solid #eee;
  text-align: left;
}

.error {
  color: #b91c1c;
}
</style>

