import SwiftUI

struct ContentView: View {
    @State private var keys: [MCCore.KeyInfo] = []
    @State private var userid = ""
    @State private var status = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack(spacing: 10) {
                Image(systemName: "lock.fill")
                    .font(.title2).foregroundStyle(.white)
                    .frame(width: 34, height: 34)
                    .background(LinearGradient(colors: [.indigo, .purple], startPoint: .top, endPoint: .bottom))
                    .clipShape(RoundedRectangle(cornerRadius: 9))
                VStack(alignment: .leading, spacing: 0) {
                    Text("Monadnock Cyber GPG").font(.headline)
                    Text("Mail extension host").font(.caption).foregroundStyle(.secondary)
                }
            }

            GroupBox("Your keys") {
                if keys.isEmpty {
                    Text("No keys found.").foregroundStyle(.secondary).frame(maxWidth: .infinity, alignment: .leading)
                } else {
                    ForEach(keys) { k in
                        HStack {
                            VStack(alignment: .leading) {
                                Text(k.userid)
                                Text(k.fingerprint).font(.system(.caption2, design: .monospaced)).foregroundStyle(.secondary)
                            }
                            Spacer()
                            Text(k.has_secret ? "secret" : "public")
                                .font(.caption2).padding(.horizontal, 7).padding(.vertical, 2)
                                .background(k.has_secret ? Color.green.opacity(0.2) : Color.gray.opacity(0.2))
                                .clipShape(Capsule())
                        }
                        .padding(.vertical, 2)
                    }
                }
            }

            GroupBox("Create a key") {
                HStack {
                    TextField("Name <email>", text: $userid)
                    Button("Generate") {
                        if let _ = MCCore.generate(userid) { status = "Created."; userid = ""; refresh() }
                        else { status = "Could not create key." }
                    }
                }
            }

            Text(status).font(.caption).foregroundStyle(.secondary)

            Text("Next: enable the extension in Mail → Settings → Privacy & Extensions.")
                .font(.caption).foregroundStyle(.secondary)
            Spacer()
        }
        .padding(20)
        .onAppear(perform: refresh)
    }

    private func refresh() { keys = MCCore.listKeys() }
}
