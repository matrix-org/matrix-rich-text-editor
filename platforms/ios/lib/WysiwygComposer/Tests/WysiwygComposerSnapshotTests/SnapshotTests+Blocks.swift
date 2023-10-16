//
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

import SnapshotTesting

final class BlocksSnapshotTests: SnapshotTests {
    func testInlineCodeContent() throws {
        viewModel.setHtmlContent("<code>test</code>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testCodeBlockContent() throws {
        viewModel.setHtmlContent("<pre><code>if snapshot {\n\treturn true\n}</code></pre>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testQuoteContent() throws {
        viewModel.setHtmlContent("<blockquote>Some quote with<br/><br/><br/><br/>line breaks inside</blockquote>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testMultipleBlocksContent() throws {
        viewModel.setHtmlContent(
            """
            <blockquote>Some<br/>\
            multi-line<br/>\
            quote</blockquote>\
            <br/>\
            <br/>\
            Some text<br/>\
            <br/>\
            <pre>A\n\tcode\nblock</pre>\
            <br/>\
            <br/>\
            Some <code>inline</code> code
            """
        )
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
}
