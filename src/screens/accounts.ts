// ======= START ACCOUNTS SCREEN =======

import { addAccount, listAccounts, logout, unlinkAccount } from "../api";
import { getAccounts, getProfile, setAccounts, setProfile } from "../state";
import { bindStepIndicator, stepIndicator } from "../components/step-indicator";
import { renderLogin } from "./login";
import { renderUninstall } from "./uninstall";

// ======= START ICONS =======

const PLUS_ICON = `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
    <line x1="12" y1="5" x2="12" y2="19"/>
    <line x1="5" y1="12" x2="19" y2="12"/>
</svg>`;

const REMOVE_ICON = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#f61e51" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
    <line x1="18" y1="6" x2="6" y2="18"/>
    <line x1="6" y1="6" x2="18" y2="18"/>
</svg>`;

const CLOSE_ICON = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
    <line x1="18" y1="6" x2="6" y2="18"/>
    <line x1="6" y1="6" x2="18" y2="18"/>
</svg>`;

// ======= END ICONS =======

// ======= START LOGOUT MODAL =======

function openLogoutModal(root: HTMLElement, onConfirm: () => void): void {
    const overlay = document.createElement("div");
    overlay.className = "modal-overlay";
    overlay.innerHTML = `
        <div class="modal" role="dialog" aria-modal="true">
            <button class="modal-close" aria-label="Close">${CLOSE_ICON}</button>
            <p class="modal-text">Log out of Discord?</p>
            <div class="modal-actions">
                <button class="btn-cancel">Cancel</button>
                <button class="btn-action btn-logout">Log Out</button>
            </div>
        </div>
    `;

    const close = () => overlay.remove();

    overlay.addEventListener("click", (e) => {
        if (e.target === overlay) close();
    });

    overlay.querySelector<HTMLButtonElement>(".modal-close")!.addEventListener("click", close);
    overlay.querySelector<HTMLButtonElement>(".btn-cancel")!.addEventListener("click", close);
    overlay.querySelector<HTMLButtonElement>(".btn-logout")!.addEventListener("click", () => {
        close();
        onConfirm();
    });

    root.appendChild(overlay);
}

// ======= END LOGOUT MODAL =======

export async function showAccounts(root: HTMLElement): Promise<void> {
    const accs = await listAccounts();
    setAccounts(accs);
    render(root);
}

function render(root: HTMLElement): void {
    const profile = getProfile();
    const accs = getAccounts();

    const avatarSrc = profile?.avatar_url ?? "";
    const avatarTitle = profile?.username ?? "";

    const accountRows = [...accs].reverse().map((a) => {
        const label = a.game_name && a.tag_line
            ? `${a.game_name}#${a.tag_line}`
            : a.puuid;
        return `
            <div class="account-row">
                <span class="account-label">${label}</span>
                <button class="btn-remove" data-puuid="${a.puuid}" aria-label="Remove account">${REMOVE_ICON}</button>
            </div>
        `;
    }).join("");

    root.innerHTML = `
        <div class="screen-accounts">
            ${stepIndicator(2)}
            <img id="discord-avatar" class="discord-avatar"
                src="${avatarSrc}" alt="${avatarTitle}" title="${avatarTitle}">
            <div class="accounts-box">
                <div class="accounts-box-header">
                    <button id="btn-add" class="btn-action" aria-label="Add account">
                        ${PLUS_ICON}
                    </button>
                </div>
                ${accs.length === 0
                    ? `<div class="accounts-empty">Click the + to add new accounts</div>`
                    : `<div class="accounts-list">${accountRows}</div>`}
            </div>
            <button id="btn-back" class="btn-action btn-back">Back</button>
            <button id="btn-finish" class="btn-action btn-finish">Finish</button>
        </div>
    `;

    bindStepIndicator(root, (step) => {
        if (step === 1) renderLogin(root);
    });

    root.querySelector<HTMLButtonElement>("#btn-back")!.addEventListener("click", () => {
        renderLogin(root);
    });

    root.querySelector<HTMLImageElement>("#discord-avatar")!.addEventListener("click", () => {
        openLogoutModal(root, async () => {
            try {
                await logout();
                setProfile(null);
                renderLogin(root);
            } catch (err) {
                console.error("logout failed:", err);
            }
        });
    });

    root.querySelector<HTMLButtonElement>("#btn-add")!.addEventListener("click", async () => {
        try {
            await addAccount();
            const accs = await listAccounts();
            setAccounts(accs);
            render(root);
        } catch (err) {
            const msg = String(err);
            // user closed the popup intentionally
            if (msg.includes("window closed before auth")) return;
            console.error("add account failed:", msg);
        }
    });

    root.querySelector<HTMLButtonElement>("#btn-finish")!.addEventListener("click", () => {
        renderUninstall(root);
    });

    root.querySelectorAll<HTMLButtonElement>(".btn-remove").forEach((btn) => {
        btn.addEventListener("click", async () => {
            const puuid = btn.dataset.puuid!;
            try {
                await unlinkAccount(puuid);
                const accs = await listAccounts();
                setAccounts(accs);
                render(root);
            } catch (err) {
                console.error("unlink failed:", err);
            }
        });
    });
}

// ======= END ACCOUNTS SCREEN =======
