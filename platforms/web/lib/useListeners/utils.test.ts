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

import { createDefaultActionStates, mapToAllActionStates } from './utils';

describe('createDefaultActionStates', () => {
    it('Should return all the action state with enabled as value', () => {
        // When
        const states = createDefaultActionStates();

        // Then
        expect(states).toStrictEqual({
            bold: 'enabled',
            italic: 'enabled',
            strikethrough: 'enabled',
            underline: 'enabled',
            clear: 'enabled',
            inlineCode: 'enabled',
            undo: 'enabled',
            redo: 'enabled',
            orderedList: 'enabled',
            unorderedList: 'enabled',
        });
    });
});

describe('mapToAllActionStates', () => {
    it('Should convert the map to an AllActionStates object', () => {
        // When
        const map = new Map([
            ['BOLD', 'eNabled'],
            ['italic', 'reversed'],
            ['uNderLine', 'DISABLED'],
        ]);
        const states = mapToAllActionStates(map);

        // Then
        expect(states).toStrictEqual({
            bold: 'enabled',
            italic: 'reversed',
            underline: 'disabled',
        });
    });
});
