// Right-click an encrypted message in webmail -> "Decrypt with Monadnock GPG"
// -> shows the plaintext in a floating panel, in place. One step.
import { decryptText } from "./lib/crypto.js";

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "mc-decrypt",
    title: "Decrypt with Monadnock GPG",
    contexts: ["selection"],
  });
});

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (info.menuItemId !== "mc-decrypt" || !info.selectionText || !tab?.id) return;
  try {
    const text = await decryptText(info.selectionText.trim());
    overlay(tab.id, "Decrypted ✓", text, true);
  } catch (e) {
    overlay(tab.id, "Couldn't decrypt", String(e?.message || e), false);
  }
});

function overlay(tabId, title, body, ok) {
  chrome.scripting.executeScript({
    target: { tabId },
    func: (t, b, ok) => {
      document.getElementById("mc-gpg-overlay")?.remove();
      const esc = (s) => s.replace(/[&<>]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;" }[c]));
      const d = document.createElement("div");
      d.id = "mc-gpg-overlay";
      d.style.cssText =
        "position:fixed;z-index:2147483647;right:18px;bottom:18px;max-width:460px;background:#131c2e;color:#e6edf7;border:1px solid #233149;border-radius:14px;padding:16px;font:13px/1.5 -apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;box-shadow:0 18px 50px rgba(0,0,0,.55)";
      d.innerHTML =
        '<div style="font-weight:700;color:' + (ok ? "#34d399" : "#f87171") + '">' + t + "</div>" +
        '<div style="margin-top:8px;white-space:pre-wrap;word-break:break-word;max-height:50vh;overflow:auto">' + esc(b) + "</div>" +
        '<div style="margin-top:10px;text-align:right">' +
        (ok ? '<button id="mc-copy" style="background:#1a2742;color:#e6edf7;border:1px solid #233149;border-radius:8px;padding:5px 12px;cursor:pointer;margin-right:8px">Copy</button>' : "") +
        '<button id="mc-close" style="background:#3b82f6;color:#fff;border:0;border-radius:8px;padding:5px 12px;cursor:pointer">Close</button></div>';
      document.body.appendChild(d);
      d.querySelector("#mc-close").onclick = () => d.remove();
      const cp = d.querySelector("#mc-copy");
      if (cp) cp.onclick = () => navigator.clipboard.writeText(b);
    },
    args: [title, body, ok],
  });
}
