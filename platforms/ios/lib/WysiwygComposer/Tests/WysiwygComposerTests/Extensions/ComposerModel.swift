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

import WysiwygComposer
import XCTest

extension ComposerModel {
    /// Execute given action that returns a `ComposerUpdate` on self.
    ///
    /// - Parameters:
    ///   - action: composer action to execute
    /// - Returns: self (discardable)
    @discardableResult
    func action(_ action: @escaping (ComposerModel) -> ComposerUpdate) -> ComposerModel {
        _ = action(self)
        return self
    }

    /// Execute given assertion (or any other code) on self.
    ///
    /// - Parameters:
    ///   - assertion: assertion/code to execute
    /// - Returns: self (discardable)
    @discardableResult
    func assert(_ assertion: @escaping (ComposerModel) -> Void) -> ComposerModel {
        assertion(self)
        return self
    }

    /// Assert given HTML matches self.
    ///
    /// - Parameters:
    ///   - html: html string to test
    /// - Returns: self (discardable)
    @discardableResult
    func assertHtml(_ html: String) -> ComposerModel {
        XCTAssertEqual(getContentAsHtml(), html)
        return self
    }

    /// Assert given tree matches self.
    ///
    /// - Parameters:
    ///   - tree: tree string to test
    /// - Returns: self (discardable)
    @discardableResult
    func assertTree(_ tree: String) -> ComposerModel {
        XCTAssertEqual(toTree(), tree)
        return self
    }
}
