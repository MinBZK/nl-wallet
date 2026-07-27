<template>
  <header class="header">
    <div>
      <h1>{{ title }}</h1>
      <p>{{ description }}</p>
    </div>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  description: string
}>()
</script>

<style scoped>
.header {
  display: flex;
  align-items: center;
  box-sizing: border-box;
  height: var(--header-height);
  padding: 15px 24px;
  border-bottom: 2px solid var(--color-border);
}

h1 {
  font-size: 24px;
  line-height: 34px;
  color: #1f2a44;
  margin-bottom: 0;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

p {
  color: var(--color-text-secondary);
  font-size: 16px;
  line-height: 22px;
  margin-top: 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
