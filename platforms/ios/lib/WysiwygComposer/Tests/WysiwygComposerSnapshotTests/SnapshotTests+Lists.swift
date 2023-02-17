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

final class ListsSnapshotTests: SnapshotTests {
    func testOrderedListContent() throws {
        viewModel.setHtmlContent("<ol><li>Item 1</li><li>Item 2</li></ol><p>Standard text</p>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testUnorderedListContent() throws {
        viewModel.setHtmlContent("<ul><li>Item 1</li><li>Item 2</li></ul><p>Standard text</p>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testMultipleListsContent() throws {
        viewModel.setHtmlContent(
            """
            <ol><li>Item 1</li><li>Item2</li></ol>\
            <ul><li>Item 1</li><li>Item2</li></ul>
            """
        )
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testIndentedListContent() throws {
        viewModel.setHtmlContent(
            """
            <ol><li>Item 1</li><li><p>Item 2</p>\
            <ol><li>Item 2A</li><li>Item 2B</li><li>Item 2C</li></ol>\
            </li><li>Item 3</li></ol>
            """
        )
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }

    func testListInQuote() throws {
        viewModel.setHtmlContent(
            """
            <blockquote>\
            <ol><li>Item 1</li><li>Item 2</li></ol>\
            <p>Some text</p>\
            </blockquote>
            """
        )
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
}
