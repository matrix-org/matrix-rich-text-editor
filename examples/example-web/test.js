import { node_and_offset } from "./wysiwyg.js";

const editor = document.getElementById('editor');

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
    }},

    { name: "ASCII characters have width 1", test: () => {
        editor.innerHTML = "abcd";
        deleteRange(0, 1);
        assert_eq(editor.innerHTML, "bcd");

        editor.innerHTML = "abcd";
        deleteRange(0, 2);
        assert_eq(editor.innerHTML, "cd");
    }},

    { name: "UCS-2 characters have width 1", test: () => {
        editor.innerHTML = "\u{03A9}bcd";
        deleteRange(0, 1);
        assert_eq(editor.innerHTML, "bcd");

        editor.innerHTML = "\u{03A9}bcd";
        deleteRange(0, 2);
        assert_eq(editor.innerHTML, "cd");
    }},

    { name: "Multi-code unit UTF-16 characters have width 2", test: () => {
        editor.innerHTML = "\u{1F4A9}bcd";
        deleteRange(0, 2);
        assert_eq(editor.innerHTML, "bcd");

        editor.innerHTML = "\u{1F4A9}bcd";
        deleteRange(0, 3);
        assert_eq(editor.innerHTML, "cd");
    }},

    { name: "Complex characters width = num UTF-16 code units", test: () => {
        editor.innerHTML = "\u{1F469}\u{1F3FF}\u{200D}\u{1F680}bcd";
        deleteRange(0, 7);
        assert_eq(editor.innerHTML, "bcd");

        editor.innerHTML = "\u{1F469}\u{1F3FF}\u{200D}\u{1F680}bcd";
        deleteRange(0, 8);
        assert_eq(editor.innerHTML, "cd");
    }}
]);

function deleteRange(start, end) {
    let textNode = editor.childNodes[0];
    const range = document.createRange();
    range.setStart(textNode, start);
    range.setEnd(textNode, end);
    var sel = document.getSelection();
    sel.removeAllRanges();
    sel.addRange(range);
    sel.deleteFromDocument();
}

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
