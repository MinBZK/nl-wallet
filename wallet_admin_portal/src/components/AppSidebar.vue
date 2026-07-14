<template>
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-icon">◼</div>
      <div>
        <div class="brand-title">NL Wallet</div>
        <div class="brand-subtitle">Acceptatie</div>
      </div>
    </div>

    <div class="nav-section">
      <div class="section-label">ALGEMEEN</div>

      <button class="nav-item active">
        <span class="icon">▦</span>
        <span>Openstaande taken</span>
      </button>

      <button class="nav-item">
        <span class="icon">⏱</span>
        <span>Taakgeschiedenis</span>
      </button>
    </div>

    <div class="user-card">
      <img class="avatar" src="https://i.pravatar.cc/80?img=12" alt="User avatar" />
      <div class="user-meta">
        <div class="user-name">Joss van Leiden</div>
        <div class="user-role">Teamleider</div>
      </div>
      <div class="chevron">›</div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  padding: 20px 16px 16px;
  background: #fff;
  border-right: 1px solid #e7e7ef;
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-bottom: 28px;
}

.brand-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: linear-gradient(135deg, #5f6bf3, #3d45d1);
  color: white;
  display: grid;
  place-items: center;
  font-size: 20px;
  box-shadow: 0 8px 24px rgba(61, 69, 209, 0.25);
}

.brand-title {
  font-weight: 700;
  font-size: 18px;
}
.brand-subtitle {
  color: #687089;
  font-size: 14px;
  margin-top: 2px;
}

.section-label {
  font-size: 12px;
  font-weight: 700;
  color: #6b7280;
  margin: 8px 0 10px;
}

.nav-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  border: 1px solid transparent;
  background: transparent;
  padding: 14px 12px;
  border-radius: 8px;
  color: #23314d;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
}

.nav-item.active {
  border-color: #5f6bf3;
  background: #f5f7ff;
  box-shadow: inset 0 0 0 1px rgba(95, 107, 243, 0.12);
}

.icon {
  width: 20px;
  text-align: center;
  color: #4b55d6;
}

.user-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 6px 4px;
  border-top: 1px solid #ececf3;
  margin-top: auto;
}

.avatar {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  object-fit: cover;
}

.user-meta {
  flex: 1;
}

.user-name {
  font-weight: 700;
  font-size: 14px;
}

.user-role {
  color: #6c7285;
  font-size: 12px;
  margin-top: 2px;
}

.chevron {
  color: #4b55d6;
  font-size: 26px;
  line-height: 1;
}
</style>
