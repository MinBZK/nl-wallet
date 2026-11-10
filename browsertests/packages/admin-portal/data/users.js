import process from "node:process"

export const TASK_ACTION_LABELS = {
  revoke_wallet: "Wallet intrekken",
  block_user: "Gebruiker blokkeren",
  unblock_user: "Gebruiker deblokkeren",
  revoke_solution: "Alle wallets intrekken",
}

export const USERS = {
  operator: {
    username: "operator1",
    password: process.env.AP_OPERATOR1_PASSWORD,
    displayName: "Oper Ator 1",
    role: "Beheerder",
    nav: { openTasks: true, myTasks: true, history: true },
    canCreateTask: true,
    taskActions: ["revoke_wallet", "block_user", "unblock_user"],
  },
  teamlead: {
    username: "teamlead1",
    password: process.env.AP_TEAMLEAD1_PASSWORD,
    displayName: "Team Lead 1",
    role: "Teamleider",
    nav: { openTasks: true, myTasks: false, history: true },
    canCreateTask: false,
    taskActions: [],
  },
  superuser: {
    username: "superuser1",
    password: process.env.AP_SUPERUSER1_PASSWORD,
    displayName: "Super User 1",
    role: "Superuser",
    nav: { openTasks: true, myTasks: true, history: true },
    canCreateTask: true,
    taskActions: ["revoke_solution"],
  },
}
