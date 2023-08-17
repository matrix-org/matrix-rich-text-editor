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

import SnapshotTesting
import SwiftUI
@testable import WysiwygComposer
import XCTest

class SnapshotTests: XCTestCase {
    let isRecord = false
    
    var viewModel = WysiwygComposerViewModel()
    var hostingController: UIViewController!
    
    override func setUpWithError() throws {
        try super.setUpWithError()
        let composerView = WysiwygComposerView(placeholder: "Placeholder",
                                               viewModel: viewModel,
                                               itemProviderHelper: nil,
                                               keyCommandHandler: nil,
                                               pasteHandler: nil)
        hostingController = UIHostingController(rootView: composerView)
    }
    
    override func tearDownWithError() throws {
        try super.tearDownWithError()
        viewModel.clearContent()
        hostingController = nil
    }
}
