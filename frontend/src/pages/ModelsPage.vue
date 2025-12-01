<script setup lang="ts">
import { ref, onMounted } from "vue";
import type { ModelFamily } from "@/types";
import { listModelFamilies } from "@/api/models";

const projectId = ref("");
const families = ref<ModelFamily[]>([]);
const loading = ref(false);
const errorMessage = ref<string | null>(null);

async function load() {
  if (!projectId.value) {
    families.value = [];
    return;
  }
  loading.value = true;
  errorMessage.value = null;
  try {
    families.value = await listModelFamilies(projectId.value);
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
    <h2>Model Families</h2>
    <form @submit.prevent="handleSubmit" class="project-form">
      <label>
        Project ID
        <input v-model="projectId" placeholder="UUID" />
      </label>
      <button type="submit">Load</button>
    </form>

    <p v-if="loading">Loading...</p>
    <p v-else-if="errorMessage" class="error">{{ errorMessage }}</p>

    <ul v-else>
      <li v-for="family in families" :key="family.id">
        <strong>{{ family.name }}</strong> — {{ family.model_type }}
      </li>
      <li v-if="families.length === 0">No model families found.</li>
    </ul>
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

.error {
  color: #b91c1c;
}
</style>

