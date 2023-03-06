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

import Foundation

/// Defines an API for permalink replacement with other objects (e.g. pills)
public protocol PermalinkReplacer {
    /// Called when the parser of the composer steps upon a link.
    /// This can be used to provide custom attributed string parts, such
    /// as a pillified representation of a link.
    /// If nothing is provided, the composer will use a standard link.
    ///
    /// - Parameters:
    ///   - link: URL of the link
    ///   - text: Text of the link
    /// - Returns: Replacement for the attributed link.
    func replacementForLink(_ link: String, text: String) -> NSAttributedString?
}
