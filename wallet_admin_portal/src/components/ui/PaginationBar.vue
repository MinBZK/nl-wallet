<template>
  <div class="pagination">
    <div class="summary">
      <span>Pagina 1 van 12</span>
      <span>Resultaten per pagina</span>
      <select>
        <option>10</option>
        <option>25</option>
        <option>50</option>
      </select>
    </div>

    <div class="pages">
      <button class="nav">
        <img src="@/assets/icons/chevron_backward.svg" alt="Previous page" />
      </button>
      <button class="page active">1</button>
      <button class="page">2</button>
      <button class="page">3</button>
      <span class="ellipsis">…</span>
      <button class="page">12</button>
      <button class="nav">
        <img src="@/assets/icons/chevron_forward.svg" alt="Next page" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-sizing: border-box;
  padding: 0.875rem 1.5rem 1.125rem;
  border-top: 2px solid var(--color-border);
}

.summary {
  display: flex;
  align-items: center;
  gap: 1rem;
  color: var(--color-text-primary);
  font-size: 0.875rem;
  line-height: 1.4286;
}

select {
  height: 38px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 0 0.75rem;
  background: #fff;
  font-size: 0.875rem;
  line-height: 1.4286;
  font-weight: 500;
  color: var(--color-text-primary);
}

.pages {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.page {
  min-width: 44px;
  height: 44px;
  border: none;
  border-radius: 12px;
  background: var(--color-surface-tint);
  color: var(--color-primary);
  font-weight: 700;
  cursor: pointer;
}

.nav {
  min-width: 36px;
  height: 36px;
  border: none;
  border-radius: 12px;
  background: var(--color-background);
  font-weight: 700;
  cursor: pointer;
  align-items: center;
  display: flex;
}

.page.active {
  background: var(--color-primary);
  color: #fff;
}

.ellipsis {
  color: var(--color-text-muted);
  padding: 0 0.375rem;
}
</style>
