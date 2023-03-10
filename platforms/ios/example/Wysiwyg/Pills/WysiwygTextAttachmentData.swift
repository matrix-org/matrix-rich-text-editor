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

import Foundation
import UIKit

/// Data associated with a Pill text attachment.
@available(iOS 15.0, *)
struct WysiwygTextAttachmentData: Codable {
    // MARK: - Properties

    /// Display name.
    var displayName: String
    /// The absolute URL for the item.
    var url: String
    /// Font for the display name
    var font: UIFont

    // MARK: - Init

    /// Init.
    ///
    /// - Parameters:
    ///   - displayName: Item display name (user or room display name)
    ///   - url: The absolute URL for the item.
    ///   - font: Font for the display name
    init(displayName: String,
         url: String,
         font: UIFont) {
        self.displayName = displayName
        self.url = url
        self.font = font
    }

    // MARK: - Codable

    enum CodingKeys: String, CodingKey {
        case displayName
        case url
        case font
    }

    enum WysiwygTextAttachmentDataError: Error {
        case noFontData
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        displayName = try container.decode(String.self, forKey: .displayName)
        url = try container.decode(String.self, forKey: .url)
        let fontData = try container.decode(Data.self, forKey: .font)
        if let font = try NSKeyedUnarchiver.unarchivedObject(ofClass: UIFont.self, from: fontData) {
            self.font = font
        } else {
            throw WysiwygTextAttachmentDataError.noFontData
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(displayName, forKey: .displayName)
        try container.encode(url, forKey: .url)
        let fontData = try NSKeyedArchiver.archivedData(withRootObject: font, requiringSecureCoding: false)
        try container.encode(fontData, forKey: .font)
    }
}
