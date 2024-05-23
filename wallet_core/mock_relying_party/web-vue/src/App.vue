<script setup lang="ts">
import 'nl-wallet-web'

function handleEvent(event: Event) {
  console.log(event)
}
</script>

<template>
  <main>
    <header>
      <div class="logo">
        <span class="logoset">
          <img src="./assets/adam_logo.svg" class="screen-logo" alt="Gemeente Amsterdam">
        </span>
        <div class="logotexts">
          <div class="logotext red">Gemeente</div>
          <div class="logotext red">Amsterdam</div>
        </div>
      </div>
    </header>

    <nav>
      <ul>
        <li><a href="#">Onderwerpen</a></li>
        <li><a href="#">Nieuws</a></li>
        <li><a href="#">Contact</a></li>
      </ul>
    </nav>

    <article>
      <h1>Leeftijd aantonen</h1>
      <div>
        <img src="./assets/70892-kloveniersburgwal-03-17apr2023-e-v-eis940.jpg" alt="Kloveniersburgwal">
      </div>
      <p class="text-xl">
        Met NL Wallet kunt u bewijzen hoe oud u bent zonder uw geboortedatum prijs te geven. U blijft dus
        anoniem.
      </p>
      <h2>Probeer het uit</h2>
      <p>
        Bewijs dat u ouder bent dan 18 jaar of 65 jaar door uzelf bekend te maken met de volgende
        gegevens:
      </p>
      <ul>
        <li>Ouder dan 18</li>
        <li>Ouder dan 65</li>
      </ul>
      <p>
        <a href="#">Waarom worden deze gegevens gevraagd?</a>
      </p>
      <div>
        <nl-wallet-button text="18+ bewijzen met NL Wallet" @close="handleEvent"></nl-wallet-button>
      </div>
    </article>
  </main>
</template>

<style scoped>

@media (min-width: 1400px) {
  main {
    max-width: 1400px;
  }
}

main {
  width: 100%;
  height: 100vh;
  margin-left: auto;
  margin-right: auto;
  padding-left: 5rem;
  padding-right: 5rem;
}

header, nav, article {
  max-width: 1536px;
  padding-left: 14rem;
  padding-right: 14rem;
  margin-left: auto;
  margin-right: auto;
}

header {
  background-color: white;
  height: 7rem;
}

nav {
  display: flex;
  align-items: center;
  height: 3rem;
  background-color: rgb(243 244 246);
  font-weight: 700;
}

article {
  background-color: white;
  padding-bottom: 5rem;
}

.screen-logo {
  height: 68px;
}

.logo {
  position: relative;
  float: left;
  margin-top: 16px;
  margin-left: -30px;
}

.logoset {
  position: absolute;
}

.logotext {
  margin-left: 28px;
  color: rgb(236, 0, 0);
  font-size: 23px;
  font-weight: 700;
  line-height: 23px;
}

nav ul {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  list-style: none;
  margin: 0;
  padding: 0;
}

nav a {
  text-decoration: none;
  color: black;
  font-weight: bolder;
  font-size: 1.1rem;
  line-height: 1rem;
}

article .text-xl {
  font-size: 1.2rem;
}

article h1 {
  margin-top: 0;
  padding: 2rem 0 0 0;
  font-size: 2rem;
}

nl-wallet-button::part(button) {
  background-color: rgb(236, 0, 0);
  border-radius: 0;
}
</style>
