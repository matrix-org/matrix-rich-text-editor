use strum_macros::AsRefStr;

#[derive(AsRefStr, Debug, PartialEq)]
pub enum ActionState {
    Enabled,
    Reversed,
    Disabled,
}
