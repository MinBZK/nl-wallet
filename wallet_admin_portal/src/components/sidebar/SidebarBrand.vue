<template>
  <div class="brand">
    <RouterLink to="/">
      <img src="@/assets/logo_nl_wallet.svg" alt="NL Wallet Logo" class="brand-logo" />
    </RouterLink>
    <div>
      <div class="brand-title">NL Wallet</div>
      <div class="brand-subtitle">Acceptatie</div>
    </div>
  </div>
</template>

<style scoped>
.brand {
  display: flex;
  align-items: center;
  gap: 1rem;
  box-sizing: border-box;
  grid-row: 1;
  border-bottom: 2px solid var(--color-border);
  padding: 1rem;
}

.brand-logo {
  width: 54px;
  height: 54px;
}

.brand-title {
  color: var(--color-text-primary);
  font-weight: 700;
  font-size: 1.25rem;
  line-height: 1.4;
}
.brand-subtitle {
  color: var(--color-text-secondary);
  font-size: 1rem;
  line-height: 1.375;
}
</style>
