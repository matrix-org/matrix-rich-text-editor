//
// Copyright 2024 The Matrix.org Foundation C.I.C
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
import UIKit

protocol ContentEquality {
    func contentEquals(other: Self) -> Bool
}

extension DomNode: ContentEquality {
    func contentEquals(other: DomNode) -> Bool {
        switch (self, other) {
        case (.container(id: _, kind: let kind, children: let children),
              .container(id: _, kind: let kindOther, children: let childrenOther)):
            return kind == kindOther && zip(children, childrenOther).allSatisfy { node, other in
                node.contentEquals(other: other)
            }
        case (.text(id: _, text: let text), .text(id: _, text: let textOther)):
            return text == textOther
        case (.lineBreak(id: _), .lineBreak(id: _)):
            return true
        case (.mention(id: _), .mention(id: _)):
            return true
        default:
            return false
        }
    }
    
    var toAttributedText: AttributedString {
        var string = AttributedString(stringLiteral: "")
        var baseAtributes = AttributeContainer()
        baseAtributes.uiKit.font = .systemFont(ofSize: 15)
        _ = attributedString(for: self, and: &string, index: 0, attributes: baseAtributes)
        return string
    }
    
    func attributedString(for node: DomNode, and string: inout AttributedString, index: Int, attributes: AttributeContainer) -> Int {
        switch node {
        case .container(id: _, kind: let kind, children: let children):
            let combinedAttributes = updateAttributes(for: kind, and: attributes)
            var returnIndex = index
            for child in children {
                returnIndex = attributedString(for: child, and: &string, index: returnIndex, attributes: combinedAttributes)
            }
            return returnIndex
        case .text(id: _, text: let text):
            string.append(AttributedString(text, attributes: attributes))
            return index + text.count
        case .lineBreak:
            string.append(AttributedString(String.lineFeed, attributes: attributes))
            return index + String.lineFeed.count
        case .mention:
            break
        }
        return index
    }
    
    func attributedString(for node: ContainerNodeKind, children: [DomNode], and string: AttributedString, index: Int) { }
    
    func updateAttributes(for kind: ContainerNodeKind, and container: AttributeContainer) -> AttributeContainer {
        var mergeContainer = AttributeContainer().merging(container)
        switch kind {
        case .generic: break
        case .formatting(let inlineStyle):
            switch inlineStyle {
            case .bold:
                if let font = container.uiKit.font {
                    var traits = font.fontDescriptor.symbolicTraits
                    traits.insert(.traitBold)
                    if let boldDescriptor = font.fontDescriptor.withSymbolicTraits(traits) {
                        let fontWithBold = UIFont(descriptor: boldDescriptor, size: font.pointSize)
                        mergeContainer.uiKit.font = fontWithBold
                    }
                }
            case .italic:
                if let font = container.uiKit.font {
                    var traits = font.fontDescriptor.symbolicTraits
                    traits.insert(.traitItalic)
                    if let italicDescriptor = font.fontDescriptor.withSymbolicTraits(traits) {
                        let fontWithItalic = UIFont(descriptor: italicDescriptor, size: font.pointSize)
                        mergeContainer.uiKit.font = fontWithItalic
                    }
                }
            case .strikeThrough:
                mergeContainer.uiKit.strikethroughStyle = .single
            case .underline:
                mergeContainer.uiKit.underlineStyle = .single
            case .inlineCode:
                if let font = container.uiKit.font {
                    var traits = font.fontDescriptor.symbolicTraits
                    traits.insert(.traitMonoSpace)
                    if let monospaceDescriptor = font.fontDescriptor.withSymbolicTraits(traits) {
                        let fontWithMonospace = UIFont(descriptor: monospaceDescriptor, size: font.pointSize)
                        mergeContainer.uiKit.font = fontWithMonospace
                    }
                }
            }
        case .link: break
        case .list: break
        case .listItem: break
        case .codeBlock: break
        case .quote: break
        case .paragraph: break
        }
        return mergeContainer
    }
}
