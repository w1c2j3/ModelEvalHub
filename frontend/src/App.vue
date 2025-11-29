<template>
  <main class="app-shell">
    <header>
      <h1>Unified Eval Platform</h1>
      <nav>
        <RouterLink to="/">Dashboard</RouterLink>
        <RouterLink to="/models">Models</RouterLink>
        <RouterLink to="/runs">Runs</RouterLink>
        <RouterLink to="/remote-tests">Remote Tests</RouterLink>
      </nav>
    </header>
    <section class="content">
      <RouterView />
    </section>
  </main>
</template>

<style scoped>
.app-shell {
  font-family: "Segoe UI", system-ui, sans-serif;
  margin: 0 auto;
  max-width: 960px;
  padding: 2rem;
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 2rem;
}

nav {
  display: flex;
  gap: 1rem;
}

a {
  text-decoration: none;
  color: #1f4df1;
}

.content {
  border: 1px solid #eee;
  border-radius: 12px;
  padding: 2rem;
  background: #fff;
}
</style>

