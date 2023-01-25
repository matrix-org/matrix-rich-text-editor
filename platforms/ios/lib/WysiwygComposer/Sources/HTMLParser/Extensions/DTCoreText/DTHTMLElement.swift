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

    func replaceNewlinesWithDiscardableElements() {
        guard let childNodes = childNodes as? [DTHTMLElement] else { return }

        if childNodes.count == 1,
           let child = childNodes.first as? DTTextHTMLElement {
            let splits = child.text().split(separator: "\n", omittingEmptySubsequences: false)
            guard splits.count > 1, splits.contains("") else { return }

            let strings: [String] = splits.map { "\($0)\n" }
            removeAllChildNodes()
            for string in strings {
                if string != "\n" {
                    var textElement = DTTextHTMLElement()
                    textElement.setText("")
                    if let lastChild = childNodes.last as? DTTextHTMLElement {
                        textElement = lastChild
                    } else {
                        addChildNode(textElement)
                        textElement.inheritAttributes(from: self)
                        textElement.interpretAttributes()
                    }
                    textElement.setText(textElement.text() + string)
                } else {
                    let newChild = DiscardableTextHTMLElement()
                    addChildNode(newChild)
                    newChild.inheritAttributes(from: self)
                    newChild.interpretAttributes()
                }
            }
        } else {
            for childNode in childNodes {
                childNode.replaceNewlinesWithDiscardableElements()
            }
        }
    }
}
