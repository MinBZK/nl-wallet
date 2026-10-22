<template>
  <div class="warning">
    <img src="@/assets/icons/warning.svg" alt="" class="icon" />
    <p><slot /></p>
  </div>
</template>

<script setup lang="ts"></script>

<style scoped>
.warning {
  display: flex;
  gap: 0.625rem;
  align-items: center;
  box-sizing: border-box;
  width: 100%;
  max-width: var(--width-task-wizard);
  padding: 1rem;
  border-radius: 8px;
  background: #fff6e8;
}

.icon {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
}

p {
  flex: 1;
  margin: 0;
  color: #000;
  font-size: 1rem;
  line-height: 1.375;
}
</style>
