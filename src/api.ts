// ======= START API =======

import { invoke } from "@tauri-apps/api/core";

export interface DiscordProfile {
    id: string;
    username: string;
    avatar_url: string;
}

export interface AddAccountResponse {
    puuid: string;
    shard: string;
    game_name?: string;
    tag_line?: string;
}

export interface AccountSummary {
    puuid: string;
    shard: string;
    game_name?: string;
    tag_line?: string;
}

export function loginDiscord(): Promise<DiscordProfile> {
    return invoke("login_discord");
}

export function addAccount(): Promise<AddAccountResponse> {
    return invoke("add_account");
}

export function listAccounts(): Promise<AccountSummary[]> {
    return invoke("list_accounts");
}

export function unlinkAccount(puuid: string): Promise<void> {
    return invoke("unlink_account", { puuid });
}

export function logout(): Promise<void> {
    return invoke("logout");
}

export function uninstall(): Promise<void> {
    return invoke("uninstall");
}

// ======= END API =======
