<script setup lang="ts">
const models = [
  { name: "Example Model", provider: "hf", runtime: "local" },
];
</script>

<template>
  <section>
    <h2>Models</h2>
    <p>Placeholder list. Wire up backend API once available.</p>
    <ul>
      <li v-for="model in models" :key="model.name">
        <strong>{{ model.name }}</strong> — {{ model.provider }} ({{ model.runtime }})
      </li>
    </ul>
  </section>
</template>

