import Foundation
import MailKit

/// Append a debug line to a file in the App Group container, which we can read
/// from outside (os_log default level isn't persisted in Release builds).
private func dbg(_ s: String) {
    guard let dir = FileManager.default.containerURL(
        forSecurityApplicationGroupIdentifier: "group.com.monadnockcyber.gpg") else { return }
    let url = dir.appendingPathComponent("decode.log")
    let line = (s + "\n").data(using: .utf8)!
    if let h = try? FileHandle(forWritingTo: url) {
        h.seekToEndOfFile(); h.write(line); try? h.close()
    } else {
        try? line.write(to: url)
    }
}

/// Principal class Mail loads. Vends the message-security handler.
@objc(MailExtension)
@MainActor
final class MailExtension: NSObject, MEExtension {
    func handlerForMessageSecurity() -> MEMessageSecurityHandler {
        SecurityHandler()
    }
}

@MainActor
final class SecurityHandler: NSObject, MEMessageSecurityHandler {

    // MARK: - Decode (incoming)

    func decodedMessage(forMessageData data: Data) -> MEDecodedMessage? {
        dbg("decode called: \(data.count) bytes")
        guard let raw = String(data: data, encoding: .utf8)
            ?? String(data: data, encoding: .ascii) else {
            dbg("not decodable as text"); return nil
        }
        guard let block = Self.extractPGPMessage(raw) else {
            dbg("no PGP block — passing through"); return nil
        }
        dbg("found PGP block (\(block.count) chars); keyring=\(MCCore.keyringPath)")

        guard let plaintext = MCCore.decrypt(block) else {
            dbg("decrypt FAILED")
            let info = MEMessageSecurityInformation(
                signers: [], isEncrypted: true, signingError: nil,
                encryptionError: NSError(domain: "MonadnockCyberGPG", code: 1, userInfo: [
                    NSLocalizedDescriptionKey: "No matching key, or message couldn't be decrypted."]))
            return MEDecodedMessage(data: nil, securityInformation: info, context: nil)
        }
        dbg("decrypted OK (\(plaintext.count) chars)")

        // Replace the ciphertext block with plaintext, keeping the original
        // headers/structure so Mail renders a valid message.
        let decoded = raw.replacingOccurrences(of: block, with: plaintext)
        let info = MEMessageSecurityInformation(
            signers: [], isEncrypted: true, signingError: nil, encryptionError: nil)
        return MEDecodedMessage(data: Data(decoded.utf8), securityInformation: info, context: nil)
    }

    // MARK: - Encode (outgoing) — v1 passthrough

    func getEncodingStatus(
        for message: MEMessage,
        composeContext: MEComposeContext,
        completionHandler: @escaping (MEOutgoingMessageEncodingStatus) -> Void
    ) {
        completionHandler(MEOutgoingMessageEncodingStatus(
            canSign: false, canEncrypt: false, securityError: nil, addressesFailingEncryption: []))
    }

    func encode(
        _ message: MEMessage,
        composeContext: MEComposeContext,
        completionHandler: @escaping (MEMessageEncodingResult) -> Void
    ) {
        completionHandler(MEMessageEncodingResult(
            encodedMessage: nil, signingError: nil, encryptionError: nil))
    }

    // MARK: - Security UI (none in v1)

    func extensionViewController(signers: [MEMessageSigner]) -> MEExtensionViewController? { nil }
    func extensionViewController(messageContext context: Data) -> MEExtensionViewController? { nil }
    func primaryActionClicked(
        forMessageContext context: Data,
        completionHandler: @escaping (MEExtensionViewController?) -> Void
    ) {
        completionHandler(nil)
    }

    private static func extractPGPMessage(_ text: String) -> String? {
        guard let start = text.range(of: "-----BEGIN PGP MESSAGE-----"),
              let end = text.range(of: "-----END PGP MESSAGE-----")
        else { return nil }
        return String(text[start.lowerBound..<end.upperBound])
    }
}
