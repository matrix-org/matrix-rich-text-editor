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

@testable import WysiwygComposer
import XCTest

extension ComposerModelWrapper {
    /// Execute given action that returns a `ComposerUpdate` on self.
    ///
    /// - Parameters:
    ///   - action: composer action to execute
    /// - Returns: self (discardable)
    @discardableResult
    func action(_ action: @escaping (ComposerModelWrapper) -> ComposerUpdate) -> ComposerModelWrapper {
        _ = action(self)
        return self
    }

    /// Execute given code on self.
    ///
    /// - Parameters:
    ///   - block: code to execute
    /// - Returns: self (discardable)
    @discardableResult
    func execute(_ block: @escaping (ComposerModelWrapper) -> Void) -> ComposerModelWrapper {
        block(self)
        return self
    }

    /// Assert given HTML matches self.
    ///
    /// - Parameters:
    ///   - html: html string to test
    /// - Returns: self (discardable)
    @discardableResult
    func assertHtml(_ html: String) -> ComposerModelWrapper {
        XCTAssertEqual(getContentAsHtml(), html)
        return self
    }

    /// Assert given tree matches self.
    ///
    /// - Parameters:
    ///   - tree: tree string to test
    /// - Returns: self (discardable)
    @discardableResult
    func assertTree(_ tree: String) -> ComposerModelWrapper {
        XCTAssertEqual(toTree(), tree)
        return self
    }

    /// Assert given selection matches self.
    ///
    /// - Parameters:
    ///   - start: selection start (UTF16 code units)
    ///   - end: selection end (UTF16 code units)
    /// - Returns: self (discardable)
    @discardableResult
    func assertSelection(start: UInt32, end: UInt32) -> ComposerModelWrapper {
        let state = getCurrentDomState()
        XCTAssertEqual(state.start, start)
        XCTAssertEqual(state.end, end)
        return self
    }

    /// Assert link action matches self.
    ///
    /// - Parameters:
    ///   - linkAction: expected link action
    /// - Returns: self (discardable)
    @discardableResult
    func assertLinkAction(_ linkAction: LinkAction) -> ComposerModelWrapper {
        XCTAssertEqual(getLinkAction(), linkAction)
        return self
    }
}
