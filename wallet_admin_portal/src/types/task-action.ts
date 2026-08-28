import { Privilege } from '@/types/privilege.ts'

/** Each task action requires the user to hold the corresponding {@link Privilege}. */
export enum TaskActionType {
  RevokeWallet = Privilege.RevokeWallet,
  BlockUser = Privilege.BlockUser,
  UnblockUser = Privilege.UnblockUser,
  RevokeSolution = Privilege.RevokeSolution,
}

/** Copy shown for a {@link TaskActionType} throughout the create-task wizard. */
export interface TaskActionInfo {
  /** Title shown throughout the create-task wizard. */
  title: string
  /** Subtext shown under the header on the wizard's first step. */
  description: string
  /** Heading shown on the wizard's first step. */
  header: string
  /** Bullet points shown on the check/confirm step. */
  consequences: string[]
}

/** Single source of truth for each task action's wizard copy, keyed by {@link TaskActionType}. */
export const taskActionInfo: Record<TaskActionType, TaskActionInfo> = {
  [TaskActionType.RevokeWallet]: {
    title: 'Wallet(s) intrekken',
    header: 'Zoek een wallet en voeg die toe aan de lijst',
    description:
      "Gebruik zoeken om een Wallet-ID te vinden. Je kunt één of meer Wallet-ID's toevoegen.",
    consequences: ['De gebruiker(s) kan/kunnen deze wallet(s) niet meer gebruiken.'],
  },
  [TaskActionType.BlockUser]: {
    title: 'Gebruiker(s) blokkeren',
    header: 'Zoek een herstelcode en voeg die toe aan de lijst',
    description:
      'Gebruik zoeken om een herstelcode te vinden. Je kunt één of meer herstelcodes toevoegen.',
    consequences: [
      'De gebruiker(s) kan/kunnen de bestaande wallet(s) niet meer gebruiken en kan/kunnen geen nieuwe wallet aanmaken.',
    ],
  },
  [TaskActionType.UnblockUser]: {
    title: 'Gebruiker(s) deblokkeren',
    header: 'Zoek een herstelcode en voeg die toe aan de lijst',
    description:
      'Gebruik zoeken om een herstelcode te vinden. Je kunt één of meer herstelcodes toevoegen.',
    consequences: ['De gebruiker(s) kunnen weer een nieuwe wallet aanmaken.'],
  },
  [TaskActionType.RevokeSolution]: {
    title: 'Alle wallets intrekken',
    header: 'Lees dit voordat je de taak aanmaakt',
    description: 'Lees eerst wanneer je deze actie gebruikt en wat het gevolg is.',
    consequences: [
      'Alle gebruikers kunnen NL Wallet meteen niet meer gebruiken.',
      'Lopende acties kunnen stoppen of mislukken.',
      'De hele Wallet Solution wordt stopgezet.',
      'Deze actie kan niet ongedaan gemaakt worden.',
    ],
  },
}
