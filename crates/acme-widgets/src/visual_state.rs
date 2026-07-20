/// Common visual states shared across widgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisualState {
    Default,
    Hover,
    Pressed,
    FocusVisible,
    Selected,
    Disabled,
    Loading,
    Invalid,
}
