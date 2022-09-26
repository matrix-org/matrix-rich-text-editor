export function isInputEvent(e: Event): e is InputEvent {
    return 'inputType' in e;
}
