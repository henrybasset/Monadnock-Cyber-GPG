import SwiftUI

@main
struct MonadnockMailApp: App {
    var body: some Scene {
        WindowGroup("Monadnock Cyber GPG — Mail") {
            ContentView()
                .frame(minWidth: 520, minHeight: 460)
        }
        .windowResizability(.contentSize)
    }
}
