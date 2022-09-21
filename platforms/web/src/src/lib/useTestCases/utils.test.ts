import init from '../../../generated/wysiwyg';
import { Actions } from './types';
import { generateTestCase, getSelectionAccordingToActions } from './utils';

beforeAll(async () => {
    await init();
});

describe('Selection', () => {
    test('Should select according to no actions is -1, 1', () => {
        // When
        const actions = [];
        const selection = getSelectionAccordingToActions(actions)();

        // Then
        expect(selection).toStrictEqual([-1, -1]);
    });

    test('Should found selection from the last action', () => {
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

describe('Generate test case', () => {
    test('Should generate test case of 1 character and selection', () => {
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

    test('Should Generate test case with cursor at the beginning', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['select', 0, 0],
        ];

        const expected = (
            'let mut model = cm("|a");\n'
            + 'assert_eq!(tx(&model), "|a");\n'
        );

        const testCase = generateTestCase(actions, 'a|');

        // Then
        expect(testCase).toBe(expected);
    });

    test('Should generate test case from multiple typed characters', () => {
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

    test('Should generate test case collecting initial selections', () => {
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

    /*
    { name: "Generate testcase collecting initial selections", test: () => {
        const actions = [
            ["replace_text", "a", undefined],
            ["select", 1, 1],
            ["replace_text", "b", undefined],
            ["select", 2, 2],
            ["replace_text", "c", undefined],
            ["select", 3, 3],
            ["replace_text", "d", undefined],
            ["select", 4, 4]
        ];

        const expected = (
            'let mut model = cm("abcd|");\n'
            + 'assert_eq!(tx(&model), "abcd|");\n'
        );

        assert_eq(expected, generate_testcase(actions, "abcd|"));
    }},

    { name: "Generate testcase with pasted start", test: () => {
        const actions = [
            ["replace_text", "abcd", undefined],
            ["select", 4, 4]
        ];

        const expected = (
            'let mut model = cm("abcd|");\n'
            + 'assert_eq!(tx(&model), "abcd|");\n'
        );

        assert_eq(expected, generate_testcase(actions, "abcd|"));
    }},

    { name: "Generate testcase by typing and bolding", test: () => {
        const actions = [
            ["replace_text", "a", undefined],
            ["replace_text", "b", undefined],
            ["replace_text", "c", undefined],
            ["replace_text", "d", undefined],
            ["select", 1, 3],
            ["bold"]
        ];

        const expected = (
            'let mut model = cm("a{bc}|d");\n'
            + 'model.bold();\n'
            + 'assert_eq!(tx(&model), "a<strong>{bc}|</strong>d");\n'
        );

        assert_eq(
            expected,
            generate_testcase(actions, "a<strong>{bc}|</strong>d")
        );
    }},

    { name: "Generate testcase with backward selection", test: () => {
        const actions = [
            ["replace_text", "a", undefined],
            ["replace_text", "b", undefined],
            ["replace_text", "c", undefined],
            ["replace_text", "d", undefined],
            ["select", 3, 1],
            ["bold"]
        ];

        const expected = (
            'let mut model = cm("a|{bc}d");\n'
            + 'model.bold();\n'
            + 'assert_eq!(tx(&model), "a<strong>|{bc}</strong>d");\n'
        );

        assert_eq(
            expected,
            generate_testcase(actions, "a<strong>|{bc}</strong>d")
        );
    }},

    { name: "Generate testcase with backward to beginning", test: () => {
        const actions = [
            ["replace_text", "a", undefined],
            ["replace_text", "b", undefined],
            ["replace_text", "c", undefined],
            ["replace_text", "d", undefined],
            ["select", 3, 0],
            ["bold"]
        ];

        const expected = (
            'let mut model = cm("|{abc}d");\n'
            + 'model.bold();\n'
            + 'assert_eq!(tx(&model), "<strong>|{abc}</strong>d");\n'
        );

        assert_eq(
            expected,
            generate_testcase(actions, "<strong>|{abc}</strong>d")
        );
    }},

    { name: "Generate testcase with backward from end", test: () => {
        const actions = [
            ["replace_text", "abc", undefined],
            ["select", 3, 2]
        ];

        const expected = (
            'let mut model = cm("ab|{c}");\n'
            + 'assert_eq!(tx(&model), "<strong>ab|{c}</strong>");\n'
        );

        assert_eq(
            expected,
            generate_testcase(actions, "<strong>ab|{c}</strong>")
        );
    }},

    { name: "Generate testcase with tags on selection boundary", test: () => {
        const actions = [
            ["replace_text", "aa<strong>bbbb</strong>cc", undefined],
            ["select", 2, 6]
        ];

        const expected = (
            'let mut model = cm("aa<strong>{bbbb}|</strong>cc");\n'
            + 'assert_eq!(tx(&model), "aa<strong>{bbbb}|</strong>cc");\n'
        );

        assert_eq(
            expected,
            generate_testcase(actions, "aa<strong>{bbbb}|</strong>cc")
        );
    }},

    { name: "Generate testcase with multiple later selections", test: () => {
        const actions = [
            ["replace_text", "aa<strong>bbbb</strong>cc", undefined],
            ["select", 2, 6],
            ["bold"],
            ["select", 3, 3],
            ["select", 3, 5],
            ["select", 4, 4],
            ["select", 3, 6]
        ];

        const expected = (
            'let mut model = cm("aa<strong>{bbbb}|</strong>cc");\n'
            + 'model.bold();\n'
            + 'model.select(Location::from(3), Location::from(6));\n'
            + 'assert_eq!(tx(&model), "aa<strong>{bbbb}|</strong>cc");\n'
        );

        assert_eq(
            expected,
            generate_testcase(actions, "aa<strong>{bbbb}|</strong>cc")
        );
    }},

    { name: "Generate testcase later selections to beginning", test: () => {
        const actions = [
            ["replace_text", "aa<strong>bbbb</strong>cc", undefined],
            ["select", 2, 6],
            ["bold"],
            ["select", 3, 0]
        ];

        const expected = (
            'let mut model = cm("aa<strong>{bbbb}|</strong>cc");\n'
            + 'model.bold();\n'
            + 'model.select(Location::from(3), Location::from(0));\n'
            + 'assert_eq!(tx(&model), "|{aa<strong>b}bbb</strong>cc");\n'
        );

        assert_eq(
            expected,
            generate_testcase(actions, "|{aa<strong>b}bbb</strong>cc")
        );
    }},

    { name: "Generate testcase multi-backspaces not suppressed", test: () => {
        const actions = [
            ["replace_text", "aa<strong>bbbb</strong>cc", undefined],
            ["select", 8, 8],
            ["backspace"],
            ["backspace"]
        ];

        const expected = (
            'let mut model = cm("aa<strong>bbbb</strong>cc|");\n'
            + 'model.backspace();\n'
            + 'model.backspace();\n'
            + 'assert_eq!(tx(&model), "aa<strong>bbbb|</strong>");\n'
        );

        assert_eq(
            expected,
            generate_testcase(actions, "aa<strong>bbbb|</strong>")
        );
    }}*/
});
