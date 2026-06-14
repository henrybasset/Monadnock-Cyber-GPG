import Foundation

/// Swift wrapper over the mc-ffi C library (the shared Rust crypto core).
enum MCCore {
    static let appGroup = "group.com.monadnockcyber.gpg"

    /// Signed builds (host app + sandboxed Mail extension) share the keyring in
    /// the App Group container. Unsigned dev builds fall back to the desktop
    /// app's keyring so existing keys still show.
    static var keyringPath: String {
        if let override = overrideKeyring { return override }
        if let group = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroup) {
            return group.appendingPathComponent("keyring").path
        }
        return NSHomeDirectory() + "/Library/Application Support/com.monadnockcyber.gpg/keyring"
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
