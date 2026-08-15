// ======= START API =======
import { invoke } from "@tauri-apps/api/core";
export function loginDiscord() {
    return invoke("login_discord");
}
export function addAccount() {
    return invoke("add_account");
}
export function listAccounts() {
    return invoke("list_accounts");
}
export function unlinkAccount(puuid) {
    return invoke("unlink_account", { puuid });
}
export function logout() {
    return invoke("logout");
}
export function uninstall() {
    return invoke("uninstall");
}
// ======= END API =======
