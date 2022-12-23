//
// Copyright 2022 The Matrix.org Foundation C.I.C
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

@testable import WysiwygComposer

import SnapshotTesting
import SwiftUI
import XCTest

final class WysiwygComposerViewSnapshotTests: XCTestCase {
    let isRecord = false
    
    var viewModel: WysiwygComposerViewModel!
    var hostingController: UIViewController!
    
    override func setUpWithError() throws {
        try super.setUpWithError()
        viewModel = WysiwygComposerViewModel()
        let binding: Binding<Bool> = .init(get: { true }, set: { _ in })
        let composerView = WysiwygComposerView(focused: binding, viewModel: viewModel)
            .placeholder("Placeholder")
        hostingController = UIHostingController(rootView: composerView)
    }
    
    func testClearState() throws {
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
    
    func testPlainTextContent() throws {
        viewModel.setHtmlContent("Test")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
    
    func testInlineCodeContent() throws {
        viewModel.setHtmlContent("<code>test</code>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
    
    func testLinkContent() throws {
        viewModel.setHtmlContent("<a href=\"https://element.io\">test</a>")
        assertSnapshot(
            matching: hostingController,
            as: .image(on: .iPhone13),
            record: isRecord
        )
    }
    
    override func tearDownWithError() throws {
        try super.tearDownWithError()
        hostingController = nil
        viewModel = nil
    }
}
