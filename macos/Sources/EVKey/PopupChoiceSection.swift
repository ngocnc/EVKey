import SwiftUI

/// Nhóm lựa chọn trong popup. `collapsible` false: tiêu đề nhỏ viết hoa và danh sách luôn hiện (Kiểu gõ).
/// `collapsible` true: một hàng hiện giá trị hiện tại, bấm để mở danh sách (Bảng mã).
struct PopupChoiceSection<Value: Hashable>: View {
    let title: String
    let choices: [ChoiceOption<Value>]
    let selected: Value
    var collapsible = true
    let onSelect: (Value) -> Void

    @State private var expanded = false

    var body: some View {
        VStack(spacing: 0) {
            if collapsible {
                Button {
                    withAnimation(Motion.slide(0.3)) { expanded.toggle() }
                } label: {
                    HStack(spacing: 8) {
                        Text(title).font(.system(size: 13))
                        Spacer(minLength: 8)
                        Text(currentLabel)
                            .font(.system(size: 12))
                            .opacity(0.65)
                        Image(systemName: "chevron.right")
                            .font(.system(size: 9, weight: .bold))
                            .opacity(0.65)
                            .rotationEffect(.degrees(expanded ? 90 : 0))
                    }
                }
                .buttonStyle(PopupRowStyle(leadingInset: 32))
                .pointerCursor()
            } else {
                Text(title.uppercased())
                    .font(.system(size: 11, weight: .semibold))
                    .tracking(0.4)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, 10)
                    .padding(.top, 6)
                    .padding(.bottom, 2)
            }

            if expanded || !collapsible {
                ForEach(choices) { choice in
                    Button {
                        onSelect(choice.value)
                    } label: {
                        HStack(spacing: 8) {
                            Image(systemName: "checkmark")
                                .font(.system(size: 11, weight: .bold))
                                .frame(width: 14)
                                .opacity(choice.value == selected ? 1 : 0)
                            Text(choice.label)
                                .font(.system(size: 13, weight: choice.value == selected ? .semibold : .regular))
                            Spacer(minLength: 8)
                        }
                    }
                    .buttonStyle(PopupRowStyle(leadingInset: collapsible ? 32 : 10))
                    .pointerCursor()
                    .transition(.opacity.combined(with: .move(edge: .top)))
                }
            }
        }
    }

    private var currentLabel: String {
        choices.first { $0.value == selected }?.label ?? ""
    }
}
