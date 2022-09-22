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
}
