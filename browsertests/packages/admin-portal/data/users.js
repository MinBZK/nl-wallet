import process from "node:process"

export const USERS = {
  operator: {
    username: "administrator",
    password: process.env.AP_OPERATOR_PASSWORD,
    displayName: "Ad Ministrator",
    role: "Beheerder",
    nav: { openTasks: true, myTasks: true, history: true },
    canCreateTask: true,
    taskActions: ["Wallet intrekken", "Gebruiker blokkeren", "Gebruiker deblokkeren"],
  },
  teamlead: {
    username: "manager",
    password: process.env.AP_TEAMLEAD_PASSWORD,
    displayName: "Ma Nager",
    role: "Teamleider",
    nav: { openTasks: true, myTasks: false, history: true },
    canCreateTask: false,
    taskActions: [],
  },
  superuser: {
    username: "destroyer",
    password: process.env.AP_SUPERUSER_PASSWORD,
    displayName: "De Stroyer",
    role: "Superuser",
    nav: { openTasks: true, myTasks: true, history: true },
    canCreateTask: true,
    taskActions: ["Alle wallets intrekken"],
  },
}
