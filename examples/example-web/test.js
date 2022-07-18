import { codepoint, node_and_offset } from "./wysiwyg.js";

run_tests([
    { name: "node_and_offset finds at the start of simple text", test: () => {
        editor.innerHTML = "abcdefgh";
        let { node, offset } = node_and_offset(editor, 0);
        assert_same(node, editor.childNodes[0]);
        assert_eq(offset, 0);
    }},

    { name: "node_and_offset finds in the middle of simple text", test: () => {
        editor.innerHTML = "abcdefgh";
        let { node, offset } = node_and_offset(editor, 4);
        assert_same(node, editor.childNodes[0]);
        assert_eq(offset, 4);
    }},

    { name: "node_and_offset finds at the end of simple text", test: () => {
        editor.innerHTML = "abcdefgh";
        let { node, offset } = node_and_offset(editor, 8);
        assert_same(node, editor.childNodes[0]);
        assert_eq(offset, 8);
    }},

    { name: "node_and_offset returns null if off the end", test: () => {
        editor.innerHTML = "abcdefgh";
        let { node, offset } = node_and_offset(editor, 9);
        assert_same(node, null);
        assert_eq(offset, 1);
    }}
]);

function run_tests(tests) {
    console.log("Running tests:");
    for (const test of tests) {
        try {
            test.test();
            console.log(` - ok - ${test.name}`);
        } catch (e) {
            console.log(` - Failed - ${test.name}`);
            throw e;
        }
    }
}

function assert_eq(left, right) {
    const le = JSON.stringify(left);
    const ri = JSON.stringify(right);
    if (le !== ri) {
        throw new Error(`${le} != ${ri}`);
    }
}

function assert_same(left, right) {
    if (left !== right) {
        throw new Error(`Assertion failed: ${left} is not ${right}`);
    }
}

/*function assert(condition, explanation) {
    if (!condition) {
        throw new Error(`Assertion failed: ${explanation}`);
    }
}*/
