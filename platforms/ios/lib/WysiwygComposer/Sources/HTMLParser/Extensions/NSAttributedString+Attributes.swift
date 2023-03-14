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

import UIKit

public extension NSAttributedString {
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

    /// Retrieve font symbolic traits at a given attributed index.
    ///
    /// - Parameters:
    ///   - index: the attributed string index to lookup
    /// - Returns: the symbolic traits at given location, empty if no font is defined
    func fontSymbolicTraits(at index: Int) -> UIFontDescriptor.SymbolicTraits {
        let font = attribute(.font, at: index, effectiveRange: nil) as? UIFont
        return font?.fontDescriptor.symbolicTraits ?? []
    }

    /// Retireve background color at a given attributed index.
    ///
    /// - Parameters:
    ///   - index: the attributed string index to lookup
    /// - Returns: the background color at given location, clear if no background color is applied
    func backgroundColor(at index: Int) -> UIColor {
        let color = attribute(.backgroundColor, at: index, effectiveRange: nil) as? UIColor
        return color ?? .clear
    }

    /// Computes whether given range or its surroundings contains
    /// a link that has been replaced with something else (e.g.: a pill)
    ///
    /// - Parameter range: the range to lookup
    /// - Returns: a boolean indicating the result
    func hasReplacementLinkNear(in range: NSRange) -> Bool {
        var hasInnerReplacement = false
        enumerateTypedAttribute(.replacementContent, in: range) { (_: ReplacementContent, _, stop) in
            hasInnerReplacement = true
            stop.pointee = true
        }
        return hasInnerReplacement
            || hasAttribute(.replacementContent, at: range.location - 1)
            || hasAttribute(.replacementContent, at: range.upperBound)
    }
}

private extension NSAttributedString {
    /// Computes whether the attributed string contains given attribute at index.
    ///
    /// - Parameters:
    ///   - attrName: the key for the attribute to test
    ///   - index: the index to lookup
    /// - Returns: a boolean indicating the result
    func hasAttribute(_ attrName: NSAttributedString.Key, at index: Int) -> Bool {
        guard index >= 0, index < length else { return false }
        return attribute(attrName, at: index, effectiveRange: nil) != nil
    }
}
