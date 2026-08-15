// ======= START STATE =======
let currentScreen = "login";
let discordProfile = null;
let accounts = [];
export function getScreen() {
    return currentScreen;
}
export function setScreen(s) {
    currentScreen = s;
}
export function getProfile() {
    return discordProfile;
}
export function setProfile(p) {
    discordProfile = p;
}
export function getAccounts() {
    return accounts;
}
export function setAccounts(a) {
    accounts = [...a];
}
// ======= END STATE =======
