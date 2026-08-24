<template>
  <div class="login-page">
    <div class="login-card">
      <div class="brand">
        <img src="@/assets/logo_nl_wallet.svg" alt="NL Wallet Logo" class="brand-logo" />
        <div class="brand-title">NL Wallet</div>
      </div>

      <h1>Log in met jouw account</h1>

      <AppButton size="lg" @click="login">
        <img src="@/assets/icons/arrow_forward.svg" alt="" class="icon" />
        <span>Inloggen</span>
      </AppButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'

function login() {
  window.location.href = '/auth/login'
}
</script>

<style scoped>
.login-page {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: #f2f1fe;
}
.login-card {
  width: 100%;
  max-width: 480px;
  box-sizing: border-box;
  background: var(--color-background);
  border-radius: 16px;
  box-shadow:
    0 4px 8px 3px rgba(21, 42, 98, 0.1),
    0 1px 3px 0 rgba(21, 42, 98, 0.15);
  padding: 40px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 40px;
  min-height: 54px;
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

h1 {
  color: var(--color-text-primary);
  font-size: 2rem;
  line-height: 1.125;
  font-weight: 700;
  margin: 0 0 2.5rem;
}

.icon {
  width: 1.25rem;
  height: 1.25rem;
}
</style>
