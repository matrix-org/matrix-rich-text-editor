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

import DTCoreText

/// Defines a placeholder to be inserted during HTML parsing in order to have a valid
/// position for e.g. an empty paragraph.
final class PlaceholderTextHTMLElement: DTTextHTMLElement {
    /// Init.
    ///
    /// - Parameters:
    ///   - textNode: text node that should be copied into the element.
    init(from textNode: DTTextHTMLElement) {
        super.init()
        setText(textNode.text())
    }

    override init() {
        super.init()
        setText(.nbsp)
    }

    override func attributesForAttributedStringRepresentation() -> [AnyHashable: Any]! {
        var dict = super.attributesForAttributedStringRepresentation() ?? [AnyHashable: Any]()
        // Insert a key to mark this as discardable post-parsing.
        dict[NSAttributedString.Key.discardableText] = true
        return dict
    }
}
