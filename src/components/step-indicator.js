// ======= START STEP INDICATOR =======
const STEPS = ["Login", "Add Accounts", "Uninstall"];
export function stepIndicator(current) {
    const steps = STEPS.map((label, i) => {
        const n = i + 1;
        const cls = n === current ? "step step-active" : n < current ? "step step-done" : "step";
        const tag = n < current ? "button" : "div";
        const attr = n < current ? ` data-step="${n}"` : "";
        return `
            <${tag} class="${cls}"${attr}>
                <span class="step-dot">${n}</span>
                <span class="step-label">${label}</span>
            </${tag}>
        `;
    }).join("");
    return `<div class="step-indicator">${steps}</div>`;
}
export function bindStepIndicator(root, onNavigate) {
    root.querySelectorAll(".step-indicator [data-step]").forEach((btn) => {
        btn.addEventListener("click", () => {
            onNavigate(Number(btn.dataset.step));
        });
    });
}
// ======= END STEP INDICATOR =======
