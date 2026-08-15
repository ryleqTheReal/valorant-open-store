// ======= START STATE =======

import type { DiscordProfile, AccountSummary } from "./api";

export type Screen = "login" | "accounts" | "uninstall";

let currentScreen: Screen = "login";
let discordProfile: DiscordProfile | null = null;
let accounts: AccountSummary[] = [];

export function getScreen(): Screen {
    return currentScreen;
}

export function setScreen(s: Screen): void {
    currentScreen = s;
}

export function getProfile(): DiscordProfile | null {
    return discordProfile;
}

export function setProfile(p: DiscordProfile | null): void {
    discordProfile = p;
}

export function getAccounts(): AccountSummary[] {
    return accounts;
}

export function setAccounts(a: AccountSummary[]): void {
    accounts = [...a];
}

// ======= END STATE =======
