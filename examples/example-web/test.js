import { node_and_offset } from "./wysiwyg.js";

const editor = document.getElementById('editor');
editor.style.display = "none";

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
    }},

    { name: "node_and_offset finds before subnode", test: () => {
        editor.innerHTML = "abc<b>def</b>gh";
        let { node, offset } = node_and_offset(editor, 2);
        assert_same(node, editor.childNodes[0]);
        assert_eq(offset, 2);
    }},

    { name: "node_and_offset finds inside subnode", test: () => {
        editor.innerHTML = "abc<b>def</b>gh";
        let { node, offset } = node_and_offset(editor, 4);
        assert_same(node, editor.childNodes[1].childNodes[0]);
        assert_eq(offset, 1);
    }},

    { name: "node_and_offset finds after subnode", test: () => {
        editor.innerHTML = "abc<b>def</b>gh";
        let { node, offset } = node_and_offset(editor, 7);
        assert_same(node, editor.childNodes[2]);
        assert_eq(offset, 1);
    }}
]);

function run_tests(tests) {
    log("Running tests:");
    for (const test of tests) {
        try {
            test.test();
            log(` - ok - ${test.name}`);
        } catch (e) {
            error(` - Failed - ${test.name}`);
            throw e;
        }
    }
}

function assert_eq(left, right) {
    const le = JSON.stringify(left);
    const ri = JSON.stringify(right);
    if (le !== ri) {
        throw_error(`${le} != ${ri}`);
    }
}

function assert_same(left, right) {
    if (left !== right) {
        throw_error(`Assertion failed: ${left} is not ${right}`);
    }
}

/*function assert(condition, explanation) {
    if (!condition) {
        throw_error(`Assertion failed: ${explanation}`);
    }
}*/

function log(msg) {
    let div = document.createElement("div");
    div.innerText = msg;
    document.body.appendChild(div);

    console.log(msg);
}

function error(msg) {
    let div = document.createElement("div");
    div.innerText = msg;
    div.style.color = "red";
    document.body.appendChild(div);

    console.error(msg);
}

function throw_error(msg) {
    let div = document.createElement("div");
    div.innerText = msg;
    div.style.color = "red";
    document.body.appendChild(div);

    throw new Error(msg);
}
