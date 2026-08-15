// ======= START UNINSTALL SCREEN =======
import { openUrl } from "@tauri-apps/plugin-opener";
import { uninstall } from "../api";
import { bindStepIndicator, stepIndicator } from "../components/step-indicator";
import { renderLogin } from "./login";
import { showAccounts } from "./accounts";
const STORE_URL = "https://store.valstats.site";
const BOT_INVITE_URL = "https://discord.com/oauth2/authorize?client_id=1484089682655580231&integration_type=0&scope=applications.commands";
export function renderUninstall(root) {
    root.innerHTML = `
        <div class="screen-uninstall">
            ${stepIndicator(3)}
            <div class="uninstall-hero">
                <h1 class="uninstall-title">This app is no longer needed and may be uninstalled now.</h1>
            </div>
            <div class="uninstall-qa">
                <div class="qa-item">
                    <button class="qa-question">
                        <span>Q: Where do I see my store now?</span>
                        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 9l6 6 6-6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                    </button>
                    <div class="qa-body">
                        <p class="qa-answer">
                            Visit <a class="uninstall-link" href="${STORE_URL}">store.valstats.site</a>
                            and log in with your Discord account. You can see the VALORANT stores of all
                            your linked accounts in the browser.
                        </p>
                    </div>
                </div>
                <div class="qa-item">
                    <button class="qa-question">
                        <span>Q: Can I use Discord instead?</span>
                        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 9l6 6 6-6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                    </button>
                    <div class="qa-body">
                        <p class="qa-answer">
                            Yes. Invite the <a class="uninstall-link" href="${BOT_INVITE_URL}">VALORANT Watcher</a>
                            bot and use these commands:
                        </p>
                        <div class="qa-commands">
                            <div class="qa-command">
                                <code>/watch add &lt;name#tag&gt; &lt;skin-name&gt;</code>
                                <span>Adds a skin to an account's watchlist. The bot DMs you once the skin appears in that account's shop.</span>
                            </div>
                            <div class="qa-command">
                                <code>/store &lt;name#tag&gt;</code>
                                <span>Shows the store of a player, as long as they also have their accounts linked.</span>
                            </div>
                            <div class="qa-command">
                                <code>/watch list</code>
                                <span>Lists all watched skins of all accounts for your Discord profile.</span>
                            </div>
                            <div class="qa-command">
                                <code>/watch remove &lt;name#tag&gt; &lt;skin-name&gt;</code>
                                <span>Removes a skin from an account's watchlist.</span>
                            </div>
                        </div>
                        <p class="qa-note">Each watchlist is individual per account.</p>
                    </div>
                </div>
                <div class="qa-item">
                    <button class="qa-question">
                        <span>Q: Why did I have to install this app?</span>
                        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 9l6 6 6-6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                    </button>
                    <div class="qa-body">
                        <p class="qa-answer">
                            What this app did is unfortunately not possible in the browser. You had to
                            install it to log in, and may now uninstall it.
                        </p>
                    </div>
                </div>
            </div>
            <button id="btn-back" class="btn-action btn-back">Back</button>
            <button id="btn-uninstall" class="btn-action btn-finish">Uninstall</button>
        </div>
    `;
    bindStepIndicator(root, (step) => {
        if (step === 1)
            renderLogin(root);
        else if (step === 2)
            showAccounts(root);
    });
    root.querySelectorAll(".qa-question").forEach((btn) => {
        btn.addEventListener("click", () => {
            btn.closest(".qa-item").classList.toggle("open");
        });
    });
    // links must open in the browser
    root.querySelectorAll(".uninstall-link").forEach((link) => {
        link.addEventListener("click", async (e) => {
            e.preventDefault();
            try {
                await openUrl(link.href);
            }
            catch (err) {
                console.error("open url failed:", err);
            }
        });
    });
    root.querySelector("#btn-back").addEventListener("click", () => {
        showAccounts(root);
    });
    root.querySelector("#btn-uninstall").addEventListener("click", async () => {
        try {
            await uninstall();
        }
        catch (err) {
            console.error("uninstall failed:", err);
        }
    });
}
// ======= END UNINSTALL SCREEN =======
