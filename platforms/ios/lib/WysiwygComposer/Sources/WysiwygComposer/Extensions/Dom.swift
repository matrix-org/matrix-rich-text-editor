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
    
    var toAttributedText: NSAttributedString {
        var string = NSMutableAttributedString(string: "")
        _ = attributedString(for: self, and: string, index: 0)
        return string
    }
    
    func attributedString(for node: DomNode, and string: NSMutableAttributedString, index: Int) -> Int {
        switch node {
        case .container(id: let id, kind: let kind, children: let children):
            var returnIndex = index
            for child in children {
                returnIndex = attributedString(for: child, and: string, index: returnIndex)
            }
            return returnIndex
        case .text(id: let id, text: let text):
            string.replaceCharacters(in: NSRange(location: index, length: 0), with: text)
            return index + text.count
        case .lineBreak(id: let id):
            string.replaceCharacters(in: NSRange(location: index, length: 0), with: String.lineFeed)
            return index + String.lineFeed.count
        case .mention(id: let id):
            break
        }
        return index
    }
    
    func attributedString(for node: ContainerNodeKind, children: [DomNode], and string: NSMutableAttributedString, index: Int) { }
}
