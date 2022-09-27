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

import init from '../../generated/wysiwyg';
import { Actions } from './types';
import { generateTestCase, getSelectionAccordingToActions } from './utils';

beforeAll(async () => {
    await init();
});

describe('getSelectionAccordingToActions', () => {
    it('Should return -1, -1 for selection when there are no actions', () => {
        // When
        const actions = [];
        const selection = getSelectionAccordingToActions(actions)();

        // Then
        expect(selection).toStrictEqual([-1, -1]);
    });

    it('Should find selection from the last action', () => {
        // When
        const actions: Actions = [
            ['foo', 'bar', 'baz'],
            ['select', 10, 10],
            ['foo', 'bar', 'baz'],
            ['select', 12, 13],
            ['foo', 'bar', 'baz'],
        ];
        const selection = getSelectionAccordingToActions(actions)();

        // Then
        expect(selection).toStrictEqual([12, 13]);
    });
});

describe('generateTestCase', () => {
    it('Should generate test case of 1 character and selection', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['select', 1, 1],
        ];

        const expected = (
            'let mut model = cm("a|");\n'
            + 'assert_eq!(tx(&model), "a|");\n'
        );

        const testCase = generateTestCase(actions, 'a|');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should Generate test case with cursor at the beginning', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['select', 0, 0],
        ];

        const expected = (
            'let mut model = cm("|a");\n'
            + 'assert_eq!(tx(&model), "|a");\n'
        );

        const testCase = generateTestCase(actions, '|a');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case from multiple typed characters', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['replace_text', 'b', undefined],
            ['replace_text', 'c', undefined],
            ['replace_text', 'd', undefined],
            ['select', 4, 4],
        ];

        const expected = (
            'let mut model = cm("abcd|");\n'
            + 'assert_eq!(tx(&model), "abcd|");\n'
        );

        const testCase = generateTestCase(actions, 'abcd|');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case collecting initial selections', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['select', 1, 1],
            ['replace_text', 'b', undefined],
            ['select', 2, 2],
            ['replace_text', 'c', undefined],
            ['select', 3, 3],
            ['replace_text', 'd', undefined],
            ['select', 4, 4],
        ];

        const expected = (
            'let mut model = cm("abcd|");\n'
            + 'assert_eq!(tx(&model), "abcd|");\n'
        );

        const testCase = generateTestCase(actions, 'abcd|');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with pasted start', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'abcd', undefined],
            ['select', 4, 4],
        ];

        const expected = (
            'let mut model = cm("abcd|");\n'
            + 'assert_eq!(tx(&model), "abcd|");\n'
        );

        const testCase = generateTestCase(actions, 'abcd|');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case by typing and bolding', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['replace_text', 'b', undefined],
            ['replace_text', 'c', undefined],
            ['replace_text', 'd', undefined],
            ['select', 1, 3],
            ['bold'],
        ];

        const expected = (
            'let mut model = cm("a{bc}|d");\n'
            + 'model.bold();\n'
            + 'assert_eq!(tx(&model), "a<strong>{bc}|</strong>d");\n'
        );

        const testCase = generateTestCase(actions, 'a<strong>{bc}|</strong>d');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with backward selection', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['replace_text', 'b', undefined],
            ['replace_text', 'c', undefined],
            ['replace_text', 'd', undefined],
            ['select', 3, 1],
            ['bold'],
        ];

        const expected = (
            'let mut model = cm("a|{bc}d");\n'
            + 'model.bold();\n'
            + 'assert_eq!(tx(&model), "a<strong>|{bc}</strong>d");\n'
        );

        const testCase = generateTestCase(actions, 'a<strong>|{bc}</strong>d');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with backward to beginning', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['replace_text', 'b', undefined],
            ['replace_text', 'c', undefined],
            ['replace_text', 'd', undefined],
            ['select', 3, 0],
            ['bold'],
        ];

        const expected = (
            'let mut model = cm("|{abc}d");\n'
            + 'model.bold();\n'
            + 'assert_eq!(tx(&model), "<strong>|{abc}</strong>d");\n'
        );

        const testCase = generateTestCase(actions, '<strong>|{abc}</strong>d');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with backward from end', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'abc', undefined],
            ['select', 3, 2],
        ];

        const expected = (
            'let mut model = cm("ab|{c}");\n'
            + 'assert_eq!(tx(&model), "<strong>ab|{c}</strong>");\n'
        );

        const testCase = generateTestCase(actions, '<strong>ab|{c}</strong>');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with tags on selection boundary', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'aa<strong>bbbb</strong>cc', undefined],
            ['select', 2, 6],
        ];

        const expected = (
            'let mut model = cm("aa<strong>{bbbb}|</strong>cc");\n'
            + 'assert_eq!(tx(&model), "aa<strong>{bbbb}|</strong>cc");\n'
        );

        const testCase = generateTestCase(actions, 'aa<strong>{bbbb}|</strong>cc');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with multiple later selections', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'aa<strong>bbbb</strong>cc', undefined],
            ['select', 2, 6],
            ['bold'],
            ['select', 3, 3],
            ['select', 3, 5],
            ['select', 4, 4],
            ['select', 3, 6],
        ];

        const expected = (
            'let mut model = cm("aa<strong>{bbbb}|</strong>cc");\n'
            + 'model.bold();\n'
            + 'model.select(Location::from(3), Location::from(6));\n'
            + 'assert_eq!(tx(&model), "aa<strong>{bbbb}|</strong>cc");\n'
        );

        const testCase = generateTestCase(actions, 'aa<strong>{bbbb}|</strong>cc');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case later selections to beginning', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'aa<strong>bbbb</strong>cc', undefined],
            ['select', 2, 6],
            ['bold'],
            ['select', 3, 0],
        ];

        const expected = (
            'let mut model = cm("aa<strong>{bbbb}|</strong>cc");\n'
            + 'model.bold();\n'
            + 'model.select(Location::from(3), Location::from(0));\n'
            + 'assert_eq!(tx(&model), "|{aa<strong>b}bbb</strong>cc");\n'
        );

        const testCase = generateTestCase(actions, '|{aa<strong>b}bbb</strong>cc');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case later selections to beginning', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'aa<strong>bbbb</strong>cc', undefined],
            ['select', 8, 8],
            ['backspace'],
            ['backspace'],
        ];

        const expected = (
            'let mut model = cm("aa<strong>bbbb</strong>cc|");\n'
            + 'model.backspace();\n'
            + 'model.backspace();\n'
            + 'assert_eq!(tx(&model), "aa<strong>bbbb|</strong>");\n'
        );

        const testCase = generateTestCase(actions, 'aa<strong>bbbb|</strong>');

        // Then
        expect(testCase).toBe(expected);
    });
});
