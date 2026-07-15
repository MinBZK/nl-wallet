<template>
  <header class="header">
    <div>
      <h1>Openstaande taken</h1>
      <p>
        Je ziet alleen taken die passen bij jouw rol en rechten. Dit zijn de taken waarvoor jij nu
        iets moet doen.
      </p>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  align-items: center;
  box-sizing: border-box;
  height: var(--header-height);
  padding: 15px 24px;
  border-bottom: 2px solid  var(--color-border);
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
