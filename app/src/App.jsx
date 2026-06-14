import { useEffect, useMemo, useState } from "react";
import {
  listKeys,
  generateKey,
  importKey,
  exportPublic,
  deleteKey,
  encryptText,
  decryptText,
  signText,
  verifyText,
  encryptFile,
  decryptFile,
  prettyFpr,
} from "./lib/api.js";

/* ---------- tiny UI primitives (shadcn-flavored, hand-rolled) ---------- */

function Button({ children, onClick, variant = "primary", disabled, className = "" }) {
  const styles = {
    primary:
      "bg-indigo-500 hover:bg-indigo-400 text-white shadow-lg shadow-indigo-500/20",
    ghost: "bg-slate-800/70 hover:bg-slate-700 text-slate-100 border border-slate-700",
    danger: "bg-rose-600/90 hover:bg-rose-500 text-white",
  };
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`rounded-lg px-4 py-2 text-sm font-semibold transition disabled:opacity-40 disabled:cursor-not-allowed ${styles[variant]} ${className}`}
    >
      {children}
    </button>
  );
}

function Card({ title, subtitle, children }) {
  return (
    <div className="rounded-2xl border border-slate-800 bg-slate-900/60 p-5">
      {title && <h3 className="text-sm font-semibold text-slate-100">{title}</h3>}
      {subtitle && <p className="mt-0.5 text-xs text-slate-400">{subtitle}</p>}
      <div className={title ? "mt-4" : ""}>{children}</div>
    </div>
  );
}

function Label({ children }) {
  return <label className="mb-1 block text-xs font-medium text-slate-400">{children}</label>;
}

const inputCls =
  "w-full rounded-lg border border-slate-700 bg-slate-950/70 px-3 py-2 text-sm text-slate-100 placeholder:text-slate-600 focus:border-indigo-500 focus:outline-none";

function Mono({ children }) {
  return (
    <textarea
      readOnly
      value={children}
      className={`${inputCls} h-40 resize-none font-mono text-xs leading-relaxed`}
    />
  );
}

/* ----------------------------- the app ----------------------------- */

const NAV = [
  { id: "keys", label: "Keys", icon: "🔑" },
  { id: "encrypt", label: "Encrypt", icon: "🔒" },
  { id: "decrypt", label: "Decrypt", icon: "🔓" },
  { id: "sign", label: "Sign", icon: "✍️" },
  { id: "files", label: "Files", icon: "📁" },
];

export default function App() {
  const [view, setView] = useState("keys");
  const [keys, setKeys] = useState([]);
  const [toast, setToast] = useState(null);

  const refresh = async () => {
    try {
      setKeys(await listKeys());
    } catch (e) {
      flash(String(e), false);
    }
  };
  useEffect(() => {
    refresh();
  }, []);

  function flash(msg, ok = true) {
    setToast({ msg, ok });
    setTimeout(() => setToast(null), 3500);
  }
  const copy = async (text, what = "Copied") => {
    await navigator.clipboard.writeText(text);
    flash(`${what} to clipboard`, true);
  };

  return (
    <div className="flex h-screen bg-slate-950 text-slate-100">
      {/* sidebar */}
      <aside className="flex w-60 shrink-0 flex-col border-r border-slate-800 bg-slate-900/50">
        <div className="flex items-center gap-3 px-5 py-5">
          <div className="grid h-9 w-9 place-items-center rounded-xl bg-gradient-to-br from-indigo-400 to-indigo-700 text-lg">
            🔒
          </div>
          <div>
            <div className="text-sm font-bold leading-tight">Monadnock</div>
            <div className="text-xs text-slate-400 leading-tight">Cyber GPG</div>
          </div>
        </div>
        <nav className="mt-2 flex-1 space-y-1 px-3">
          {NAV.map((n) => (
            <button
              key={n.id}
              onClick={() => setView(n.id)}
              className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition ${
                view === n.id
                  ? "bg-indigo-500/15 text-indigo-300"
                  : "text-slate-400 hover:bg-slate-800/60 hover:text-slate-200"
              }`}
            >
              <span>{n.icon}</span>
              {n.label}
            </button>
          ))}
        </nav>
        <div className="px-5 py-4 text-[11px] text-slate-600">
          {keys.length} key{keys.length === 1 ? "" : "s"} · v0.2 · local-only
        </div>
      </aside>

      {/* main */}
      <main
        className="flex-1 overflow-y-auto"
        style={{
          background:
            "radial-gradient(900px 500px at 80% -10%, rgba(79,70,229,0.18), transparent 60%)",
        }}
      >
        <div className="mx-auto max-w-2xl px-8 py-8">
          {view === "keys" && (
            <KeysView keys={keys} refresh={refresh} flash={flash} copy={copy} />
          )}
          {view === "encrypt" && <EncryptView keys={keys} flash={flash} copy={copy} />}
          {view === "decrypt" && <DecryptView keys={keys} flash={flash} copy={copy} />}
          {view === "sign" && <SignView keys={keys} flash={flash} copy={copy} />}
          {view === "files" && <FilesView keys={keys} flash={flash} />}
        </div>
      </main>

      {toast && (
        <div
          className={`fixed bottom-5 left-1/2 -translate-x-1/2 rounded-xl px-4 py-2.5 text-sm font-medium shadow-xl ${
            toast.ok ? "bg-emerald-600 text-white" : "bg-rose-600 text-white"
          }`}
        >
          {toast.msg}
        </div>
      )}
    </div>
  );
}

function Header({ title, subtitle }) {
  return (
    <div className="mb-6">
      <h1 className="text-2xl font-bold">{title}</h1>
      <p className="mt-1 text-sm text-slate-400">{subtitle}</p>
    </div>
  );
}

function KeysView({ keys, refresh, flash, copy }) {
  const [userid, setUserid] = useState("");
  const [importing, setImporting] = useState("");
  const [busy, setBusy] = useState(false);

  const create = async () => {
    if (!userid.trim()) return flash("Enter a name and email first.", false);
    setBusy(true);
    try {
      const k = await generateKey(userid.trim());
      flash(`Created key for ${k.userid}`, true);
      setUserid("");
      refresh();
    } catch (e) {
      flash(String(e), false);
    } finally {
      setBusy(false);
    }
  };

  const doImport = async () => {
    if (!importing.trim()) return;
    try {
      const k = await importKey(importing.trim());
      flash(`Imported ${k.userid}`, true);
      setImporting("");
      refresh();
    } catch (e) {
      flash(String(e), false);
    }
  };

  const share = async (fpr) => {
    try {
      copy(await exportPublic(fpr), "Public key copied");
    } catch (e) {
      flash(String(e), false);
    }
  };

  const remove = async (fpr) => {
    try {
      await deleteKey(fpr);
      flash("Key deleted", true);
      refresh();
    } catch (e) {
      flash(String(e), false);
    }
  };

  return (
    <>
      <Header title="Keys" subtitle="Create or import keys. Your secret keys never leave this Mac." />

      <div className="space-y-4">
        <Card title="Create a new key" subtitle="A modern Curve25519 OpenPGP key, ready to use.">
          <Label>Your name &amp; email</Label>
          <div className="flex gap-2">
            <input
              className={inputCls}
              placeholder="Jane Doe <jane@example.com>"
              value={userid}
              onChange={(e) => setUserid(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && create()}
            />
            <Button onClick={create} disabled={busy}>
              {busy ? "Generating…" : "Generate"}
            </Button>
          </div>
        </Card>

        <Card title="Your keys">
          {keys.length === 0 ? (
            <p className="text-sm text-slate-500">
              No keys yet — create your first one above. 👆
            </p>
          ) : (
            <ul className="space-y-2">
              {keys.map((k) => (
                <li
                  key={k.fingerprint}
                  className="flex items-center justify-between rounded-xl border border-slate-800 bg-slate-950/50 px-4 py-3"
                >
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="truncate text-sm font-medium">{k.userid}</span>
                      <span
                        className={`rounded-full px-2 py-0.5 text-[10px] font-semibold ${
                          k.has_secret
                            ? "bg-emerald-500/15 text-emerald-300"
                            : "bg-slate-700/50 text-slate-400"
                        }`}
                      >
                        {k.has_secret ? "secret" : "public"}
                      </span>
                    </div>
                    <div className="mt-0.5 truncate font-mono text-[11px] text-slate-500">
                      {prettyFpr(k.fingerprint)}
                    </div>
                  </div>
                  <div className="flex shrink-0 gap-2 pl-3">
                    <Button variant="ghost" onClick={() => share(k.fingerprint)}>
                      Share
                    </Button>
                    <Button variant="danger" onClick={() => remove(k.fingerprint)}>
                      Delete
                    </Button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </Card>

        <Card title="Import a key" subtitle="Paste someone's public key (or your own backup).">
          <textarea
            className={`${inputCls} h-28 resize-none font-mono text-xs`}
            placeholder="-----BEGIN PGP PUBLIC KEY BLOCK-----"
            value={importing}
            onChange={(e) => setImporting(e.target.value)}
          />
          <div className="mt-3">
            <Button variant="ghost" onClick={doImport}>
              Import
            </Button>
          </div>
        </Card>
      </div>
    </>
  );
}

function EncryptView({ keys, flash, copy }) {
  const [recipient, setRecipient] = useState("");
  const [text, setText] = useState("");
  const [out, setOut] = useState("");

  useEffect(() => {
    if (!recipient && keys.length) setRecipient(keys[0].fingerprint);
  }, [keys]); // eslint-disable-line

  const run = async () => {
    if (!recipient) return flash("Choose a recipient.", false);
    if (!text) return flash("Type a message to encrypt.", false);
    try {
      setOut(await encryptText(text, recipient));
      flash("Encrypted ✓", true);
    } catch (e) {
      flash(String(e), false);
    }
  };

  return (
    <>
      <Header title="Encrypt" subtitle="Scramble a message so only the recipient can read it." />
      {keys.length === 0 ? (
        <Card>
          <p className="text-sm text-slate-500">Add a key first (Keys tab) to encrypt to someone.</p>
        </Card>
      ) : (
        <div className="space-y-4">
          <Card>
            <Label>Recipient</Label>
            <select
              className={inputCls}
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
            >
              {keys.map((k) => (
                <option key={k.fingerprint} value={k.fingerprint}>
                  {k.userid}
                </option>
              ))}
            </select>
            <div className="mt-4">
              <Label>Message</Label>
              <textarea
                className={`${inputCls} h-32 resize-none`}
                placeholder="Type your secret message…"
                value={text}
                onChange={(e) => setText(e.target.value)}
              />
            </div>
            <div className="mt-4">
              <Button onClick={run}>Encrypt</Button>
            </div>
          </Card>

          {out && (
            <Card title="Encrypted message" subtitle="Safe to send over email, chat, anywhere.">
              <Mono>{out}</Mono>
              <div className="mt-3">
                <Button variant="ghost" onClick={() => copy(out, "Ciphertext copied")}>
                  Copy
                </Button>
              </div>
            </Card>
          )}
        </div>
      )}
    </>
  );
}

function DecryptView({ flash, copy }) {
  const [cipher, setCipher] = useState("");
  const [out, setOut] = useState("");

  const run = async () => {
    if (!cipher.trim()) return flash("Paste an encrypted message.", false);
    try {
      setOut(await decryptText(cipher.trim()));
      flash("Decrypted ✓", true);
    } catch (e) {
      flash(String(e), false);
    }
  };

  return (
    <>
      <Header title="Decrypt" subtitle="Unlock a message encrypted to one of your keys." />
      <div className="space-y-4">
        <Card>
          <Label>Encrypted message</Label>
          <textarea
            className={`${inputCls} h-40 resize-none font-mono text-xs`}
            placeholder="-----BEGIN PGP MESSAGE-----"
            value={cipher}
            onChange={(e) => setCipher(e.target.value)}
          />
          <div className="mt-3">
            <Button onClick={run}>Decrypt</Button>
          </div>
        </Card>
        {out && (
          <Card title="Decrypted message">
            <textarea
              readOnly
              value={out}
              className={`${inputCls} h-32 resize-none`}
            />
            <div className="mt-3">
              <Button variant="ghost" onClick={() => copy(out, "Text copied")}>
                Copy
              </Button>
            </div>
          </Card>
        )}
      </div>
    </>
  );
}

function SignView({ keys, flash, copy }) {
  const [signer, setSigner] = useState("");
  const [text, setText] = useState("");
  const [out, setOut] = useState("");
  const [toVerify, setToVerify] = useState("");
  const [result, setResult] = useState(null);

  const signerKeys = keys.filter((k) => k.has_secret);
  useEffect(() => {
    if (!signer && signerKeys.length) setSigner(signerKeys[0].fingerprint);
  }, [keys]); // eslint-disable-line

  const doSign = async () => {
    if (!signer) return flash("Choose a key to sign with.", false);
    if (!text) return flash("Type something to sign.", false);
    try {
      setOut(await signText(text, signer));
      flash("Signed ✓", true);
    } catch (e) {
      flash(String(e), false);
    }
  };
  const doVerify = async () => {
    if (!toVerify.trim()) return flash("Paste a signed message.", false);
    try {
      setResult(await verifyText(toVerify.trim()));
    } catch (e) {
      setResult(null);
      flash(String(e), false);
    }
  };

  return (
    <>
      <Header title="Sign & Verify" subtitle="Prove a message is from you — or check who signed one." />
      <div className="space-y-4">
        <Card title="Sign a message">
          {signerKeys.length === 0 ? (
            <p className="text-sm text-slate-500">
              You need one of your own keys (create one in Keys).
            </p>
          ) : (
            <>
              <Label>Sign with</Label>
              <select className={inputCls} value={signer} onChange={(e) => setSigner(e.target.value)}>
                {signerKeys.map((k) => (
                  <option key={k.fingerprint} value={k.fingerprint}>
                    {k.userid}
                  </option>
                ))}
              </select>
              <div className="mt-4">
                <Label>Message</Label>
                <textarea
                  className={`${inputCls} h-28 resize-none`}
                  placeholder="Type a message to sign…"
                  value={text}
                  onChange={(e) => setText(e.target.value)}
                />
              </div>
              <div className="mt-4">
                <Button onClick={doSign}>Sign</Button>
              </div>
              {out && (
                <div className="mt-4">
                  <Mono>{out}</Mono>
                  <div className="mt-3">
                    <Button variant="ghost" onClick={() => copy(out, "Signed message copied")}>
                      Copy
                    </Button>
                  </div>
                </div>
              )}
            </>
          )}
        </Card>

        <Card title="Verify a signed message">
          <textarea
            className={`${inputCls} h-32 resize-none font-mono text-xs`}
            placeholder="-----BEGIN PGP MESSAGE-----"
            value={toVerify}
            onChange={(e) => setToVerify(e.target.value)}
          />
          <div className="mt-3">
            <Button variant="ghost" onClick={doVerify}>
              Verify
            </Button>
          </div>
          {result &&
            (result.valid ? (
              <div className="mt-4 rounded-xl border border-emerald-700/40 bg-emerald-500/10 p-4">
                <div className="text-sm font-semibold text-emerald-300">✓ Valid signature</div>
                <div className="mt-1 text-xs text-slate-300">
                  Signed by {result.signer || "an unknown key"}
                </div>
                <div className="mt-3 whitespace-pre-wrap rounded-lg bg-slate-950/60 p-3 text-sm">
                  {result.text}
                </div>
              </div>
            ) : (
              <div className="mt-4 rounded-xl border border-rose-700/40 bg-rose-500/10 p-4 text-sm font-semibold text-rose-300">
                ✗ Couldn't verify — unknown signer or the message was changed.
              </div>
            ))}
        </Card>
      </div>
    </>
  );
}

function FilesView({ keys, flash }) {
  const [recipient, setRecipient] = useState("");
  const [busy, setBusy] = useState(false);
  const [last, setLast] = useState(null);

  useEffect(() => {
    if (!recipient && keys.length) setRecipient(keys[0].fingerprint);
  }, [keys]); // eslint-disable-line

  const enc = async () => {
    if (!recipient) return flash("Choose a recipient.", false);
    setBusy(true);
    try {
      const out = await encryptFile(recipient);
      if (out) {
        setLast({ kind: "Encrypted", path: out });
        flash("File encrypted ✓", true);
      }
    } catch (e) {
      flash(String(e), false);
    } finally {
      setBusy(false);
    }
  };
  const dec = async () => {
    setBusy(true);
    try {
      const out = await decryptFile();
      if (out) {
        setLast({ kind: "Decrypted", path: out });
        flash("File decrypted ✓", true);
      }
    } catch (e) {
      flash(String(e), false);
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <Header
        title="Files"
        subtitle="Encrypt or decrypt files on your Mac — pick a file, the result is saved next to it."
      />
      <div className="space-y-4">
        <Card title="Encrypt a file">
          {keys.length === 0 ? (
            <p className="text-sm text-slate-500">Add a key first (Keys tab).</p>
          ) : (
            <>
              <Label>Recipient</Label>
              <select className={inputCls} value={recipient} onChange={(e) => setRecipient(e.target.value)}>
                {keys.map((k) => (
                  <option key={k.fingerprint} value={k.fingerprint}>
                    {k.userid}
                  </option>
                ))}
              </select>
              <div className="mt-4">
                <Button onClick={enc} disabled={busy}>
                  {busy ? "Working…" : "Choose file & encrypt"}
                </Button>
              </div>
              <p className="mt-2 text-xs text-slate-500">
                Saves an encrypted copy as <span className="font-mono">name.ext.pgp</span> next to the original.
              </p>
            </>
          )}
        </Card>

        <Card title="Decrypt a file">
          <Button variant="ghost" onClick={dec} disabled={busy}>
            {busy ? "Working…" : "Choose file & decrypt"}
          </Button>
          <p className="mt-2 text-xs text-slate-500">
            Decrypts a <span className="font-mono">.pgp</span> file with a secret key in your keyring.
          </p>
        </Card>

        {last && (
          <div className="rounded-xl border border-emerald-700/40 bg-emerald-500/10 p-4">
            <div className="text-sm font-semibold text-emerald-300">{last.kind} ✓</div>
            <div className="mt-1 break-all font-mono text-xs text-slate-300">{last.path}</div>
          </div>
        )}
      </div>
    </>
  );
}
