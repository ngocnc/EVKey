import SwiftUI

/// Collapsible popup picker: the header shows the current value; the list marks the selected choice.
struct PopupChoiceSection<Value: Hashable>: View {
    let title: String
    let choices: [ChoiceOption<Value>]
    let selected: Value
    let onSelect: (Value) -> Void

    @State private var expanded = false

    var body: some View {
        VStack(spacing: 0) {
            Button {
                withAnimation(.easeInOut(duration: 0.2)) { expanded.toggle() }
            } label: {
                HStack(spacing: 8) {
                    Text(title)
                        .font(.system(size: 11, weight: .semibold))
                        .tracking(0.4)
                        .foregroundStyle(.secondary)
                    Spacer(minLength: 8)
                    Text(currentLabel)
                        .font(.system(size: 12))
                        .id(currentLabel)
                        .transition(.opacity)
                        .animation(.easeInOut(duration: 0.15), value: currentLabel)
                    Image(systemName: "chevron.right")
                        .font(.system(size: 9, weight: .bold))
                        .foregroundStyle(.secondary)
                        .rotationEffect(.degrees(expanded ? 90 : 0))
                        .animation(.easeInOut(duration: 0.2), value: expanded)
                }
            }
            .buttonStyle(PopupRowStyle())

            if expanded {
                ForEach(choices) { choice in
                    Button {
                        onSelect(choice.value)
                    } label: {
                        HStack(spacing: 8) {
                            Text(choice.label)
                                .font(.system(size: 13, weight: choice.value == selected ? .semibold : .regular))
                            Spacer(minLength: 8)
                            Image(systemName: "checkmark")
                                .font(.system(size: 11, weight: .semibold))
                                .foregroundStyle(Color.accentColor)
                                .opacity(choice.value == selected ? 1 : 0)
                                .animation(.easeInOut(duration: 0.15), value: selected)
                        }
                    }
                    .buttonStyle(PopupRowStyle(leadingInset: 26))
                    .transition(.opacity)
                }
            }
        }
    }

    private var currentLabel: String {
        choices.first { $0.value == selected }?.label ?? ""
    }
}
