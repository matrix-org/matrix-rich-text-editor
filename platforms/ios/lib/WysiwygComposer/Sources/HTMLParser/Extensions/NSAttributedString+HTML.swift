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

import Foundation
import UIKit

/// Describe an error occurring during HTML string build.
public enum BuildHtmlAttributedError: LocalizedError, Equatable {
    /// Encoding data from raw HTML input failed.
    case dataError(encoding: String.Encoding)

    public var errorDescription: String? {
        switch self {
        case let .dataError(encoding: encoding):
            return "Unable to encode string with: \(encoding.description) rawValue: \(encoding.rawValue)"
        }
    }
}

public extension NSAttributedString {
    /// Init with HTML string.
    ///
    /// - Parameters:
    ///   - html: Raw HTML string.
    ///   - encoding: Character encoding to use. Default: .utf16.
    convenience init(html: String, encoding: String.Encoding = .utf16) throws {
        let attributed = try HTMLParser.parse(html: html,
                                              encoding: .utf16,
                                              textColor: UIColor.label,
                                              linkColor: UIColor.link,
                                              codeBackgroundColor: UIColor.secondarySystemBackground)
        self.init(attributedString: attributed)
    }
}
