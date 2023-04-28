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

extension DTHTMLElement {
    /// Sanitize the DTHTMLElement right before it's written inside the resulting attributed string.
    func sanitize() {
        guard let childNodes = childNodes as? [DTHTMLElement] else { return }

        if childNodes.count == 1, let child = childNodes.first as? DTTextHTMLElement {
            if child.text() == .nbsp {
                // Removing NBSP character from e.g. <p>&nbsp;</p> since it is only used to
                // make DTCoreText able to easily parse new lines.
                removeAllChildNodes()
                let newChild = PlaceholderTextHTMLElement(from: child)
                addChildNode(newChild)
                newChild.inheritAttributes(from: self)
                newChild.interpretAttributes()
            } else {
                if tag == .code, parent().tag == .pre, var text = child.text() {
                    // Replace leading and trailing NBSP from code blocks with
                    // discardable elements (ZWSP).
                    let hasLeadingNbsp = text.hasPrefix(String.nbsp)
                    let hasTrailingNbsp = text.hasSuffix(String.nbsp)
                    guard hasLeadingNbsp || hasTrailingNbsp else { return }
                    removeAllChildNodes()
                    if hasLeadingNbsp {
                        text.removeFirst()
                        addChildNode(createDiscardableElement())
                    }
                    addChildNode(child)
                    if hasTrailingNbsp {
                        text.removeLast()
                        if text.last == .lineFeed {
                            text.removeLast()
                            addChildNode(createLineBreak())
                        }
                        addChildNode(createDiscardableElement())
                    }
                    child.setText(text)
                }
            }
        } else {
            childNodes.forEach { $0.sanitize() }
        }
    }
}

// MARK: - Helpers

/// An arbitrary enum of HTML tags that requires some specific handling
private enum DTHTMLElementTag: String {
    case pre
    case code
}

private extension DTHTMLElement {
    var tag: DTHTMLElementTag? {
        DTHTMLElementTag(rawValue: name)
    }

    func createDiscardableElement() -> PlaceholderTextHTMLElement {
        let discardableElement = PlaceholderTextHTMLElement()
        discardableElement.inheritAttributes(from: self)
        discardableElement.interpretAttributes()
        return discardableElement
    }

    func createLineBreak() -> DTBreakHTMLElement {
        let lineBreakElement = DTBreakHTMLElement()
        lineBreakElement.inheritAttributes(from: self)
        lineBreakElement.interpretAttributes()
        return lineBreakElement
    }
}
