/*
Copyright 2022 The Matrix.org Foundation C.I.C.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

import { ACTION_TYPES } from '../constants';
import { ActionTypes, ActionState, AllActionStates } from '../types';

/**
 * Create the default state for all the available actions
 * @returns {AllActionStates}
 */
export function createDefaultActionStates(): AllActionStates {
    return ACTION_TYPES.reduce<AllActionStates>((acc, action) => {
        acc[action] = 'enabled';
        return acc;
    }, {} as AllActionStates);
}

/**
 * Convert a Map<string, string> containing title-case strings like:
 * "Bold": "Enabled"
 * to a AllActionStates record with entries like:
 * bold: enabled
 * @param {Map<string, string>} actionStatesMap Map to convert
 * @returns {AllActionStates}
 */
export function mapToAllActionStates(
    actionStatesMap: Map<string, string>,
): AllActionStates {
    const ret = {} as AllActionStates;
    for (const [key, value] of actionStatesMap) {
        switch (key) {
            case 'OrderedList':
                ret.orderedList = value.toLowerCase() as ActionState;
                break;
            case 'UnorderedList':
                ret.unorderedList = value.toLowerCase() as ActionState;
                break;
            case 'InlineCode':
                ret.inlineCode = value.toLowerCase() as ActionState;
                break;
            default:
                ret[key.toLowerCase() as ActionTypes] =
                    value.toLowerCase() as ActionState;
        }
    }
    return ret as AllActionStates;
}
