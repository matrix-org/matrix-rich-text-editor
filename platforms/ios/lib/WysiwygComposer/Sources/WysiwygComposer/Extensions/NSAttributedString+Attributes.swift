//
// Copyright 2022 The Matrix.org Foundation C.I.C
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

import UIKit

extension NSAttributedString {
    /// Retrieves character at given attributed index.
    ///
    /// - Parameters:
    ///   - index: the attributed string index to lookup
    /// - Returns: the character at given location
    func character(at index: Int) -> Character? {
        guard index < length else { return nil }
        let substring = attributedSubstring(from: .init(location: index, length: 1))
        return substring.string.first
    }
    
    /// Enumerate attribute for given key and conveniently ignore any attribute that doesn't match given generic type.
    ///
    /// - Parameters:
    ///   - attrName: The name of the attribute to enumerate.
    ///   - enumerationRange: The range over which the attribute values are enumerated. If omitted, the entire range is used.
    ///   - opts: The options used by the enumeration. For possible values, see NSAttributedStringEnumerationOptions.
    ///   - block: The block to apply to ranges of the specified attribute in the attributed string.
    func enumerateTypedAttribute<T>(_ attrName: NSAttributedString.Key,
                                    in enumerationRange: NSRange? = nil,
                                    options opts: NSAttributedString.EnumerationOptions = [],
                                    using block: (T, NSRange, UnsafeMutablePointer<ObjCBool>) -> Void) {
        enumerateAttribute(attrName,
                           in: enumerationRange ?? .init(location: 0, length: length),
                           options: opts) { (attr: Any?, range: NSRange, stop: UnsafeMutablePointer<ObjCBool>) in
            guard let typedAttr = attr as? T else { return }
            
            block(typedAttr, range, stop)
        }
    }
    
    /// Retrieve font symbolic traits at a given attributed index.
    ///
    /// - Parameters:
    ///   - index: the attributed string index to lookup
    /// - Returns: the symbolic traits at givem location, empty if no font is defined
    func fontSymbolicTraits(at index: Int) -> UIFontDescriptor.SymbolicTraits {
        let font = attribute(.font, at: index, effectiveRange: nil) as? UIFont
        return font?.fontDescriptor.symbolicTraits ?? []
    }

    /// Check if all characters from given range are newlines.
    ///
    /// - Parameters:
    ///   - range: the range to check for newlines
    /// - Returns: true if all characters are newlines, false otherwise
    func isAllNewlines(in range: NSRange) -> Bool {
        (range.location..<range.upperBound).allSatisfy { self.character(at: $0)?.isNewline == true }
    }
    
    /// Changes the attribute of foregroundColor for the whole attributed string
    ///
    /// - Parameters:
    ///   - color: the new UIColor to update the attributed string
    ///   - linkColor: the new UIColor for links inside the attributed string
    ///   - codeBackgroundColor: color to apply to the background of code blocks
    /// - Returns: a new attributed string with the same content and attributes, but its foregroundColor is changed
    func changeColor(to color: UIColor, linkColor: UIColor, codeBackgroundColor: UIColor) -> NSAttributedString {
        let mutableAttributed = NSMutableAttributedString(attributedString: self)
        mutableAttributed.addAttributes(
            [.foregroundColor: color], range: NSRange(location: 0, length: mutableAttributed.length)
        )
        
        // This fixes an iOS bug where if some text is typed after a link, and then a whitespace is added the link color is overridden.
        mutableAttributed.enumerateAttribute(.link, in: NSRange(location: 0, length: mutableAttributed.length)) { value, range, _ in
            if value != nil {
                mutableAttributed.addAttributes([.foregroundColor: linkColor], range: range)
            }
        }
        
        // This a temporary workaround, since inline code should not contain newlines
        // iOS removes the background color when a newline is added
        // This will be removed when inline code will avoid newlines
        mutableAttributed.enumerateTypedAttribute(.font) { (font: UIFont, range, _) in
            if font.fontDescriptor.symbolicTraits.contains(.traitMonoSpace) {
                mutableAttributed.addAttributes([.backgroundColor: codeBackgroundColor], range: range)
            }
        }

        // Fix quotes middle newlines not having the required background by manually applying it.
        // Right ow this doesn't handle multiple occurences of quotes or code blocks that are only
        // separated by newlines since we can't distinguish text from within a quote block and regular
        // text at the moment when we are at the attributed string level.
        // FIXME: improve this to handle recoloring better.
        var previous: (color: UIColor, range: NSRange)?
        mutableAttributed.enumerateTypedAttribute(.backgroundColor) { (color: UIColor, range, _) in
            if let previous = previous,
               color == previous.color,
               let midrange = NSRange(between: previous.range, and: range),
               isAllNewlines(in: midrange) {
                mutableAttributed.addAttribute(.backgroundColor, value: color, range: midrange)
            }

            if mutableAttributed.character(at: range.upperBound)?.isNewline == true {
                previous = (color, range)
            } else {
                previous = nil
            }
        }

        let newSelf = NSAttributedString(attributedString: mutableAttributed)
        return newSelf
    }
}
