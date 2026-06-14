import Foundation

/// Swift wrapper over the mc-ffi C library (the shared Rust crypto core).
enum MCCore {
    /// For the (unsandboxed) host app: share the desktop app's keyring so your
    /// existing keys appear. The sandboxed Mail extension will instead use an
    /// App Group container (set MC_KEYRING_OVERRIDE there).
    static var keyringPath: String {
        if let override = overrideKeyring { return override }
        let home = NSHomeDirectory()
        return home + "/Library/Application Support/com.monadnockcyber.gpg/keyring"
    }
    static var overrideKeyring: String?

    private static func take(_ ptr: UnsafeMutablePointer<CChar>?) -> String? {
        guard let ptr else { return nil }
        defer { mc_string_free(ptr) }
        return String(cString: ptr)
    }

    static func decrypt(_ ciphertext: String) -> String? {
        take(mc_decrypt(keyringPath, ciphertext))
    }
    static func encrypt(_ text: String, to fingerprint: String) -> String? {
        take(mc_encrypt(keyringPath, text, fingerprint))
    }
    static func sign(_ text: String, as fingerprint: String) -> String? {
        take(mc_sign(keyringPath, text, fingerprint))
    }
    static func generate(_ userid: String) -> String? {
        take(mc_generate(keyringPath, userid))
    }
    static func importKey(_ armored: String) -> String? {
        take(mc_import(keyringPath, armored))
    }

    struct KeyInfo: Decodable, Identifiable {
        let fingerprint: String
        let userid: String
        let has_secret: Bool
        var id: String { fingerprint }
    }

    static func listKeys() -> [KeyInfo] {
        guard let json = take(mc_list_json(keyringPath)),
              let data = json.data(using: .utf8),
              let keys = try? JSONDecoder().decode([KeyInfo].self, from: data)
        else { return [] }
        return keys
    }
}
