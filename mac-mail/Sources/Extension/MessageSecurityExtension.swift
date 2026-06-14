import Foundation
import MailKit

/// Principal class Mail loads. It vends the message-security handler.
@objc(MailExtension)
@MainActor
final class MailExtension: NSObject, MEExtension {
    func handlerForMessageSecurity() -> MEMessageSecurityHandler {
        SecurityHandler()
    }
}

/// Decrypts inline-PGP messages inside Mail via the shared Rust core (mc-ffi).
/// v1: incoming decryption. Outgoing encryption and PGP/MIME come next.
@MainActor
final class SecurityHandler: NSObject, MEMessageSecurityHandler {

    // MARK: - Decode (incoming mail)

    func decodedMessage(forMessageData data: Data) -> MEDecodedMessage? {
        let raw = String(data: data, encoding: .utf8) ?? String(data: data, encoding: .ascii)
        guard let raw,
              let block = Self.extractPGPMessage(raw),
              let plaintext = MCCore.decrypt(block)
        else {
            return nil // not an encrypted message we can read — let Mail proceed
        }

        let mime = "Content-Type: text/plain; charset=utf-8\r\n"
            + "Content-Transfer-Encoding: 8bit\r\n\r\n"
            + plaintext
        let info = MEMessageSecurityInformation(
            signers: [], isEncrypted: true, signingError: nil, encryptionError: nil
        )
        return MEDecodedMessage(data: Data(mime.utf8), securityInformation: info, context: nil)
    }

    // MARK: - Encode (outgoing mail) — v1 passthrough

    func getEncodingStatus(
        for message: MEMessage,
        composeContext: MEComposeContext,
        completionHandler: @escaping (MEOutgoingMessageEncodingStatus) -> Void
    ) {
        completionHandler(
            MEOutgoingMessageEncodingStatus(
                canSign: false, canEncrypt: false, securityError: nil, addressesFailingEncryption: []
            )
        )
    }

    func encode(
        _ message: MEMessage,
        composeContext: MEComposeContext,
        completionHandler: @escaping (MEMessageEncodingResult) -> Void
    ) {
        completionHandler(
            MEMessageEncodingResult(encodedMessage: nil, signingError: nil, encryptionError: nil)
        )
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
