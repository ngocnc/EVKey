import EVKeyKit
import SwiftUI

/// Chips for `customConsonants`: each letter removable, plus a field and '+' button to add one. Shown only while the checkbox is on.
struct SettingsConsonantChips: View {
    @ObservedObject private var state = AppState.shared
    @State private var draft = ""
    @State private var error: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 72), spacing: 6, alignment: .leading)], alignment: .leading, spacing: 6) {
                ForEach(state.settings.customConsonants, id: \.self) { letter in
                    chip(letter)
                }
            }
            .animation(.easeInOut(duration: 0.2), value: state.settings.customConsonants)

            HStack(spacing: 8) {
                TextField("Phụ âm", text: $draft)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 80)
                    .onSubmit(add)
                Button(action: add) {
                    Image(systemName: "plus")
                        .font(.system(size: 11, weight: .bold))
                        .foregroundStyle(.white)
                        .frame(width: 24, height: 24)
                        .background(Color.blue, in: Circle())
                }
                    .pointerCursor()
                .buttonStyle(.plain)
            }
            if let error {
                Text(error)
                    .font(.system(size: 12))
                    .foregroundStyle(.red)
            }
            Text("Các phụ âm này sẽ được phép sử dụng trong bộ gõ Tiếng Việt")
                .font(.system(size: 11))
                .foregroundStyle(.secondary)
        }
        .padding(14)
    }

    private func chip(_ letter: String) -> some View {
        HStack(spacing: 4) {
            Text(letter.uppercased())
                .font(.system(size: 13, weight: .bold, design: .monospaced))
            Button {
                state.removeCustomConsonant(letter)
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
                .pointerCursor()
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 5)
        .background(Color.primary.opacity(0.09), in: Capsule())
    }

    private func add() {
        error = state.addCustomConsonant(draft)
        if error == nil { draft = "" }
    }
}
