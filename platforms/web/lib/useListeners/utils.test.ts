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

import {
    createDefaultActionStates,
    mapToAllActionStates,
    nodeIsMention,
} from './utils';

describe('createDefaultActionStates', () => {
    it('Should return all the action state with enabled as value', () => {
        // When
        const states = createDefaultActionStates();

        // Then
        expect(states).toStrictEqual({
            bold: 'enabled',
            italic: 'enabled',
            strikeThrough: 'enabled',
            underline: 'enabled',
            clear: 'enabled',
            inlineCode: 'enabled',
            undo: 'enabled',
            redo: 'enabled',
            orderedList: 'enabled',
            unorderedList: 'enabled',
            link: 'enabled',
            codeBlock: 'enabled',
            quote: 'enabled',
            indent: 'enabled',
            unindent: 'enabled',
        });
    });
});

describe('mapToAllActionStates', () => {
    it('Should convert the map to an AllActionStates object', () => {
        // When
        const map = new Map([
            ['Bold', 'eNabled'],
            ['Italic', 'reversed'],
            ['Underline', 'DISABLED'],
            ['InlineCode', 'ENABLED'],
            ['OrderedList', 'ENABLED'],
            ['UnorderedList', 'ENABLED'],
            ['StrikeThrough', 'ENABLED'],
        ]);
        const states = mapToAllActionStates(map);

        // Then
        expect(states).toStrictEqual({
            bold: 'enabled',
            italic: 'reversed',
            underline: 'disabled',
            inlineCode: 'enabled',
            orderedList: 'enabled',
            unorderedList: 'enabled',
            strikeThrough: 'enabled',
        });
    });
});

describe('nodeIsMention', () => {
    it('should return false if node has multiple children', () => {
        const input = document.createElement('span');
        input.appendChild(document.createElement('img'));
        input.appendChild(document.createElement('span'));

        expect(nodeIsMention(input)).toBe(false);
    });

    // eslint-disable-next-line max-len
    it('should return false if node is missing the data-mention-type attribute', () => {
        const input = document.createElement('a');
        input.appendChild(document.createTextNode('text'));
        input.setAttribute('contenteditable', 'false');

        expect(nodeIsMention(input)).toBe(false);
    });

    // eslint-disable-next-line max-len
    it('should return false if node is missing the contenteditable attribute', () => {
        const input = document.createElement('a');
        input.appendChild(document.createTextNode('text'));
        input.setAttribute('data-mention-type', 'user');

        expect(nodeIsMention(input)).toBe(false);
    });

    it('should return true if node is shaped like a mention', () => {
        const input = document.createElement('a');
        input.appendChild(document.createTextNode('text'));
        input.setAttribute('contenteditable', 'false');
        input.setAttribute('data-mention-type', 'user');

        expect(nodeIsMention(input)).toBe(true);
    });
});
