import { ComposerUpdate } from "../../generated/wysiwyg";

// TODO move
const actions: Array<[string, any, any]> = [];

export function action(update: ComposerUpdate | null, name: string, value1?: any, value2?: any) {
    if (value2 !== undefined) {
        console.debug(`composer_model.${name}(${value1}, ${value2})`);
    } else if (value1 !== undefined) {
        console.debug(`composer_model.${name}(${value1})`);
    } else {
        console.debug(`composer_model.${name}()`);
    }

    actions.push([name, value1, value2]);

    // TODO test case
    // update_testcase(update);

    return update;
}

export function getSelectionAccordingToActions() {
    for (let i = actions.length - 1; i >= 0; i--) {
        const action = actions[i];
        if (action[0] === "select") {
            return [action[1], action[2]];
        }
    }
    return [-1, -1];
}
