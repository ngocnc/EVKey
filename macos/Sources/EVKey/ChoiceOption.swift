/// A labelled value for segmented controls and popup pickers.
struct ChoiceOption<Value: Hashable>: Identifiable {
    let value: Value
    let label: String

    var id: Value { value }
}
