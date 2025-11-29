<script setup lang="ts">
import { ref } from "vue";

const runs = ref([
  {
    id: "run_001",
    experiment: "baseline-mmlu",
    status: "queued",
  },
]);
</script>

<template>
  <section>
    <h2>Runs</h2>
    <table>
      <thead>
        <tr>
          <th>Run ID</th>
          <th>Experiment</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="run in runs" :key="run.id">
          <td>{{ run.id }}</td>
          <td>{{ run.experiment }}</td>
          <td>{{ run.status }}</td>
        </tr>
      </tbody>
    </table>
  </section>
</template>

<style scoped>
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
</style>

