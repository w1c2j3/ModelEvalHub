<script setup lang="ts">
import { ref } from "vue";
import { triggerRemoteTest } from "@/api/tests";

const projectId = ref("");
const description = ref("");
const statusMessage = ref<string | null>(null);
const loading = ref(false);

async function submit() {
  if (!projectId.value) {
    statusMessage.value = "Project ID is required";
    return;
  }
  loading.value = true;
  statusMessage.value = null;
  try {
    await triggerRemoteTest({
      project_id: projectId.value,
      description: description.value || undefined,
    });
    statusMessage.value = "Remote test request accepted.";
  } catch (err) {
    statusMessage.value = (err as Error).message;
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <section>
    <h2>Remote Tests</h2>
    <p>Use this form to trigger the remote test placeholder endpoint.</p>

    <form @submit.prevent="submit" class="remote-form">
      <label>
        Project ID
        <input v-model="projectId" placeholder="UUID" />
      </label>
      <label>
        Description
        <textarea v-model="description" rows="3" placeholder="Optional notes" />
      </label>
      <button type="submit" :disabled="loading">
        {{ loading ? "Submitting..." : "Trigger Remote Test" }}
      </button>
    </form>

    <p v-if="statusMessage" class="status">{{ statusMessage }}</p>
  </section>
</template>

<style scoped>
.remote-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  max-width: 480px;
}

input,
textarea {
  width: 100%;
  padding: 0.5rem;
  border-radius: 8px;
  border: 1px solid #d0d7e3;
}

button {
  padding: 0.6rem;
  border: none;
  border-radius: 8px;
  background: #2563eb;
  color: white;
  cursor: pointer;
}

button:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.status {
  margin-top: 1rem;
  color: #0f172a;
}
</style>

