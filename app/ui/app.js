// Phase 0 frontend — talks to the Rust core via Tauri commands.
const { invoke } = window.__TAURI__.core;

let currentKey = null; // { public, secret, fingerprint }

function setStatus(id, msg, ok) {
  const el = document.getElementById(id);
  el.textContent = msg;
  el.className = "status" + (ok === true ? " ok" : ok === false ? " err" : "");
}

document.getElementById("genBtn").onclick = async () => {
  setStatus("genStatus", "Generating…");
  try {
    currentKey = await invoke("generate_key", {
      userid: document.getElementById("userid").value,
    });
    const f = document.getElementById("fpr");
    f.style.display = "inline-block";
    f.textContent = "Fingerprint: " + currentKey.fingerprint;
    setStatus("genStatus", "Key generated and held in memory.", true);
  } catch (e) {
    setStatus("genStatus", String(e), false);
  }
};

document.getElementById("encBtn").onclick = async () => {
  if (!currentKey) return setStatus("encStatus", "Generate a key first.", false);
  setStatus("encStatus", "Encrypting…");
  try {
    const ct = await invoke("encrypt", {
      plaintext: document.getElementById("plain").value,
      recipientPublic: currentKey.public,
    });
    document.getElementById("cipher").value = ct;
    setStatus("encStatus", "Encrypted ✓", true);
  } catch (e) {
    setStatus("encStatus", String(e), false);
  }
};

document.getElementById("decBtn").onclick = async () => {
  if (!currentKey) return setStatus("decStatus", "Generate a key first.", false);
  setStatus("decStatus", "Decrypting…");
  try {
    const pt = await invoke("decrypt", {
      ciphertext: document.getElementById("cipher").value,
      secret: currentKey.secret,
    });
    document.getElementById("decrypted").value = pt;
    setStatus("decStatus", "Decrypted ✓ — round-trip works!", true);
  } catch (e) {
    setStatus("decStatus", String(e), false);
  }
};
