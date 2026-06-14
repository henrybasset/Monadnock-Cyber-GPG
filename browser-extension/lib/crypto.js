// OpenPGP operations for the extension, with a keyring in chrome.storage.local.
// Interoperable with the desktop app (same OpenPGP standard).
import * as openpgp from "./openpgp.min.mjs";

const KEY = "keyring";

async function load() {
  const got = await chrome.storage.local.get(KEY);
  return got[KEY] || [];
}
async function save(kr) {
  await chrome.storage.local.set({ [KEY]: kr });
}

async function describe(armored) {
  let key;
  let hasSecret = false;
  try {
    key = await openpgp.readPrivateKey({ armoredKey: armored });
    hasSecret = true;
  } catch {
    key = await openpgp.readKey({ armoredKey: armored });
  }
  const userId = key.getUserIDs()[0] || "(no user id)";
  return { fingerprint: key.getFingerprint().toUpperCase(), userId, hasSecret, armored };
}

async function readPub(entry) {
  if (entry.hasSecret) {
    return (await openpgp.readPrivateKey({ armoredKey: entry.armored })).toPublic();
  }
  return openpgp.readKey({ armoredKey: entry.armored });
}

export async function generateKey(name, email) {
  const { privateKey } = await openpgp.generateKey({
    type: "ecc",
    curve: "curve25519",
    userIDs: [{ name, email }],
    format: "armored",
  });
  return importKey(privateKey);
}

export async function importKey(armored) {
  const entry = await describe(armored.trim());
  const kr = await load();
  const idx = kr.findIndex((k) => k.fingerprint === entry.fingerprint);
  if (idx >= 0) {
    // keep the secret version if we already hold it
    if (!(kr[idx].hasSecret && !entry.hasSecret)) kr[idx] = entry;
  } else {
    kr.push(entry);
  }
  await save(kr);
  return { fingerprint: entry.fingerprint, userId: entry.userId, hasSecret: entry.hasSecret };
}

export async function listKeys() {
  return (await load()).map(({ fingerprint, userId, hasSecret }) => ({
    fingerprint,
    userId,
    hasSecret,
  }));
}

export async function deleteKey(fingerprint) {
  await save((await load()).filter((k) => k.fingerprint !== fingerprint));
}

export async function exportPublic(fingerprint) {
  const e = (await load()).find((k) => k.fingerprint === fingerprint);
  if (!e) throw new Error("key not found");
  return (await readPub(e)).armor();
}

export async function encryptText(text, recipientFpr) {
  const e = (await load()).find((k) => k.fingerprint === recipientFpr);
  if (!e) throw new Error("recipient not found");
  const message = await openpgp.createMessage({ text });
  return openpgp.encrypt({ message, encryptionKeys: await readPub(e) });
}

export async function decryptText(armoredMessage) {
  const secret = (await load()).filter((k) => k.hasSecret);
  if (!secret.length) throw new Error("no secret keys in this extension's keyring — import yours");
  const decryptionKeys = await Promise.all(
    secret.map((e) => openpgp.readPrivateKey({ armoredKey: e.armored }))
  );
  const message = await openpgp.readMessage({ armoredMessage });
  const { data } = await openpgp.decrypt({ message, decryptionKeys });
  return data;
}
