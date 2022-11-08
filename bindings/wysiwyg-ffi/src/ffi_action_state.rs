use strum_macros::AsRefStr;

#[derive(AsRefStr)]
pub enum ActionState {
    Enabled,
    Reversed,
    Disabled,
}
