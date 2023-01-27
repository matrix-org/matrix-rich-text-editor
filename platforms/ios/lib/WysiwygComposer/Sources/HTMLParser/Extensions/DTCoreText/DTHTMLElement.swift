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
            let newChild = DiscardableTextHTMLElement(from: child)
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
            var leadingDiscardableElement: DiscardableTextHTMLElement?
            var trailingDiscardableElement: DiscardableTextHTMLElement?
            var shouldReplaceNodes = false

            if text.hasPrefix("\(Character.nbsp)") {
                shouldReplaceNodes = true
                text.removeFirst()
                leadingDiscardableElement = createDiscardableElement()
            }
            
            if text.hasSuffix("\(Character.nbsp)") {
                shouldReplaceNodes = true
                text.removeLast()
                trailingDiscardableElement = createDiscardableElement()
            }

            if shouldReplaceNodes {
                removeAllChildNodes()

                if let leadingDiscardableElement = leadingDiscardableElement {
                    addChildNode(leadingDiscardableElement)
                    addChildNode(createLineBreak())
                }

                let newTextNode = DTTextHTMLElement()
                newTextNode.inheritAttributes(from: self)
                newTextNode.interpretAttributes()
                newTextNode.setText(text)
                addChildNode(newTextNode)

                if let trailingDiscardableElement = trailingDiscardableElement {
                    addChildNode(createLineBreak())
                    addChildNode(trailingDiscardableElement)
                }
            }
        } else {
            for childNode in childNodes {
                childNode.clearTrailingAndLeadingNewlinesInCodeblocks()
            }
        }
    }

    private func createDiscardableElement() -> DiscardableTextHTMLElement {
        let discardableElement = DiscardableTextHTMLElement()
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
