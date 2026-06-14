import {
  generateKey,
  importKey,
  listKeys,
  deleteKey,
  exportPublic,
  encryptText,
  decryptText,
} from "./lib/crypto.js";

const $ = (id) => document.getElementById(id);
const pretty = (fpr) => (fpr.match(/.{1,4}/g) || []).join(" ");

function status(id, msg, ok) {
  const el = $(id);
  el.textContent = msg;
  el.className = "status" + (ok === true ? " ok" : ok === false ? " err" : "");
}

// tabs
document.querySelectorAll(".tab").forEach((t) => {
  t.onclick = () => {
    document.querySelectorAll(".tab").forEach((x) => x.classList.remove("active"));
    t.classList.add("active");
    ["keys", "encrypt", "decrypt"].forEach((name) =>
      $("tab-" + name).classList.toggle("hide", name !== t.dataset.tab)
    );
  };
});

async function refresh() {
  const keys = await listKeys();
  // key list
  const list = $("keylist");
  list.innerHTML = keys.length
    ? ""
    : '<div class="hint" style="margin-top:8px">No keys yet — generate or import one.</div>';
  for (const k of keys) {
    const row = document.createElement("div");
    row.className = "keyitem";
    row.innerHTML =
      '<div style="min-width:0"><div style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap">' +
      k.userId +
      ' <span class="badge ' + (k.hasSecret ? "secret" : "public") + '">' +
      (k.hasSecret ? "secret" : "public") +
      '</span></div><div class="fpr">' + pretty(k.fingerprint) + "</div></div>";
    const actions = document.createElement("div");
    actions.style.cssText = "display:flex;gap:6px;flex-shrink:0";
    const share = document.createElement("button");
    share.className = "ghost";
    share.textContent = "Share";
    share.style.margin = "0";
    share.onclick = async () => {
      await navigator.clipboard.writeText(await exportPublic(k.fingerprint));
      share.textContent = "Copied!";
      setTimeout(() => (share.textContent = "Share"), 1200);
    };
    const del = document.createElement("button");
    del.textContent = "✕";
    del.style.cssText = "margin:0;background:#3a1f2b;border:1px solid #5b2230;color:#f5a3b3";
    del.onclick = async () => {
      await deleteKey(k.fingerprint);
      refresh();
    };
    actions.append(share, del);
    row.append(actions);
    list.append(row);
  }
  // recipient dropdown
  const sel = $("enc-recipient");
  sel.innerHTML = "";
  for (const k of keys) {
    const o = document.createElement("option");
    o.value = k.fingerprint;
    o.textContent = k.userId;
    sel.append(o);
  }
}

$("gen-btn").onclick = async () => {
  const name = $("gen-name").value.trim();
  const email = $("gen-email").value.trim();
  if (!name && !email) return status("gen-status", "Enter a name or email.", false);
  status("gen-status", "Generating…");
  try {
    const k = await generateKey(name, email);
    status("gen-status", "Created " + k.userId, true);
    $("gen-name").value = $("gen-email").value = "";
    refresh();
  } catch (e) {
    status("gen-status", String(e.message || e), false);
  }
};

$("import-btn").onclick = async () => {
  const text = $("import-text").value.trim();
  if (!text) return;
  try {
    const k = await importKey(text);
    status("import-status", "Imported " + k.userId, true);
    $("import-text").value = "";
    refresh();
  } catch (e) {
    status("import-status", String(e.message || e), false);
  }
};

$("enc-btn").onclick = async () => {
  const recipient = $("enc-recipient").value;
  const text = $("enc-text").value;
  if (!recipient) return status("enc-status", "Add a key first.", false);
  if (!text) return status("enc-status", "Type a message.", false);
  try {
    const ct = await encryptText(text, recipient);
    $("enc-out").value = ct;
    $("enc-out").classList.remove("hide");
    $("enc-copy").classList.remove("hide");
    status("enc-status", "Encrypted ✓", true);
  } catch (e) {
    status("enc-status", String(e.message || e), false);
  }
};
$("enc-copy").onclick = () => navigator.clipboard.writeText($("enc-out").value);

$("dec-btn").onclick = async () => {
  const text = $("dec-text").value.trim();
  if (!text) return status("dec-status", "Paste an encrypted message.", false);
  try {
    const pt = await decryptText(text);
    $("dec-out").value = pt;
    $("dec-out").classList.remove("hide");
    $("dec-copy").classList.remove("hide");
    status("dec-status", "Decrypted ✓", true);
  } catch (e) {
    status("dec-status", String(e.message || e), false);
  }
};
$("dec-copy").onclick = () => navigator.clipboard.writeText($("dec-out").value);

refresh();
