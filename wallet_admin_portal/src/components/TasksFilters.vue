<template>
  <section class="filters">
    <select class="select">
      <option>Alle open taken (105)</option>
      <option>Mijn taken</option>
      <option>Hoogste prioriteit</option>
    </select>

    <input class="search" type="text" placeholder="Zoek op taak-ID, Wallet-ID of herstelcode" />

    <button class="search-btn">Zoeken</button>
  </section>
</template>

<style scoped>
.filters {
  display: grid;
  grid-template-columns: 320px 1fr 110px;
  gap: 14px;
  padding: 24px;
  align-items: center;
}

.select,
.search {
  height: 44px;
  border: 1px solid #e4e6ef;
  border-radius: 6px;
  background: #fff;
  padding: 0 14px;
  font-size: 14px;
  color: #2a3550;
  outline: none;
}

.search::placeholder {
  color: #9aa1b2;
}

.search-btn {
  height: 44px;
  border: none;
  border-radius: 6px;
  background: #4b55d6;
  color: #fff;
  font-weight: 700;
  font-size: 14px;
  cursor: pointer;
}

.search-btn:hover {
  background: #404bcc;
}
</style>
