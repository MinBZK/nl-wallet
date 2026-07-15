<template>
  <div class="brand">
    <img src="@/assets/logo_nl_wallet.svg" alt="NL Wallet Logo" class="brand-logo" />
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
  gap: 16px;
  padding-bottom: 16px;
}

.brand-logo {
  width: 54px;
  height: 54px;
}

.brand-title {
  color: #152A62;
  font-weight: 700;
  font-size: 20px;
  line-height: 28px
}
.brand-subtitle {
  color: #445581;
  font-size: 16px;
  line-height: 22px
}
</style>
