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
      <button class="nav">‹</button>
      <button class="page active">1</button>
      <button class="page">2</button>
      <button class="page">3</button>
      <span class="ellipsis">…</span>
      <button class="page">12</button>
      <button class="nav">›</button>
    </div>
  </footer>
</template>

<style scoped>
.pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 24px 18px;
  border-top: 1px solid #e7e7ef;
  background: #fff;
  margin-top: auto;
}

.summary {
  display: flex;
  align-items: center;
  gap: 18px;
  color: #47506a;
  font-size: 14px;
}

select {
  height: 38px;
  border: 1px solid #e4e6ef;
  border-radius: 6px;
  padding: 0 12px;
  background: #fff;
  color: #39445f;
}

.pages {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page,
.nav {
  min-width: 42px;
  height: 42px;
  border: none;
  border-radius: 12px;
  background: #f3f4fb;
  color: #4b55d6;
  font-weight: 700;
  cursor: pointer;
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
