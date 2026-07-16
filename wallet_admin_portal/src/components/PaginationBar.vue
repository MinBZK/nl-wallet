<template>
  <footer class="pagination">
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
        <img src="@/assets/icons/chevron_backward.svg" alt="Previous page"/>
      </button>
      <button class="page active">1</button>
      <button class="page">2</button>
      <button class="page">3</button>
      <span class="ellipsis">…</span>
      <button class="page">12</button>
      <button class="nav">
        <img src="@/assets/icons/chevron_forward.svg" alt="Next page"/>
      </button>
    </div>
  </footer>
</template>

<style scoped>
.pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-sizing: border-box;
  height: var(--sidebar-footer-height);
  padding: 14px 24px 18px;
  border-top: 2px solid  var(--color-border);
  margin-top: auto;
}

.summary {
  display: flex;
  align-items: center;
  gap: 16px;
  color: var(--color-text-primary);
  font-size: 14px;
  line-height: 20px;
}

select {
  height: 38px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 0 12px;
  background: #fff;
  font-size: 14px;
  line-height: 20px;
  font-weight: 500;
  color: #1A0006;
}

.pages {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page {
  min-width: 44px;
  height: 44px;
  border: none;
  border-radius: 12px;
  background: #f3f4fb;
  color: #4b55d6;
  font-weight: 700;
  cursor: pointer;
}

.nav {
  min-width: 36px;
  height: 36px;
  border: none;
  border-radius: 12px;
  background: --var(--color-background);
  font-weight: 700;
  cursor: pointer;
  align-items: center;
  display: flex;
}

.page.active {
  background: #4b55d6;
  color: #fff;
}

.ellipsis {
  color: #6c7285;
  padding: 0 6px;
}
</style>
