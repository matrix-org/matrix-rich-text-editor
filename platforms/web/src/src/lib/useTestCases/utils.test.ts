import init from "../../../generated/wysiwyg";
import { Actions } from "./types";
import { generateTestCase } from "./utils";

describe('Generate test case', () => {
    beforeAll(async () => {
        await init();
    });

    test('Should generate test case of 1 character and selection', async () => {
        // When
        const actions: Actions = [
            ["replace_text", "a", undefined],
            ["select", 1, 1],
        ];

        const expected = (
            'let mut model = cm("a|");\n'
            + 'assert_eq!(tx(&model), "a|");\n'
        );

        // Then
        expect(generateTestCase(actions, "a|")).toBe(expected);
    });
});
