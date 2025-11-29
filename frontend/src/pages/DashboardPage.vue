<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useHealthApi } from "@/api/health";

const status = ref("Loading...");
const api = useHealthApi();

onMounted(async () => {
  try {
    status.value = await api.check();
  } catch (err) {
    status.value = `Error: ${(err as Error).message}`;
  }
});
</script>

<template>
  <section>
    <h2>System Status</h2>
    <p>{{ status }}</p>
  </section>
</template>

