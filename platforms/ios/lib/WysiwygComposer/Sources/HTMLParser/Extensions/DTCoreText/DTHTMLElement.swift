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
    /// Clears any only child consisting of a single non-breaking space.
    func clearNbspNodes() {
        guard let childNodes = childNodes as? [DTHTMLElement] else { return }

        if childNodes.count == 1,
           let child = childNodes.first as? DTTextHTMLElement,
           child.text() == .nbsp {
            removeAllChildNodes()
            let newChild = PlaceholderTextHTMLElement(from: child)
            addChildNode(newChild)
            newChild.inheritAttributes(from: self)
            newChild.interpretAttributes()
        } else {
            for childNode in childNodes {
                childNode.clearNbspNodes()
            }
        }
    }

    func clearTrailingAndLeadingNewlinesInCodeblocks() {
        guard let childNodes = childNodes as? [DTHTMLElement] else {
            return
        }

        if name == "pre",
           childNodes.count == 1,
           let child = childNodes.first as? DTTextHTMLElement,
           var text = child.text(),
           text != .nbsp {
            let hasLeadingNbsp = text.hasPrefix(String.nbsp)
            let hasTrailingNbsp = text.hasSuffix(String.nbsp)
            guard hasLeadingNbsp || hasTrailingNbsp else { return }
            removeAllChildNodes()
            if hasLeadingNbsp {
                text.removeFirst()
                addChildNode(createDiscardableElement())
                addChildNode(createLineBreak())
            }
            addChildNode(child)
            if hasTrailingNbsp {
                text.removeLast()
                addChildNode(createLineBreak())
                addChildNode(createDiscardableElement())
            }
            child.setText(text)
        } else {
            for childNode in childNodes {
                childNode.clearTrailingAndLeadingNewlinesInCodeblocks()
            }
        }
    }

    private func createDiscardableElement() -> PlaceholderTextHTMLElement {
        let discardableElement = PlaceholderTextHTMLElement()
        discardableElement.inheritAttributes(from: self)
        discardableElement.interpretAttributes()
        return discardableElement
    }

    private func createLineBreak() -> DTBreakHTMLElement {
        let lineBreakElement = DTBreakHTMLElement()
        lineBreakElement.inheritAttributes(from: self)
        lineBreakElement.interpretAttributes()
        return lineBreakElement
    }
}
