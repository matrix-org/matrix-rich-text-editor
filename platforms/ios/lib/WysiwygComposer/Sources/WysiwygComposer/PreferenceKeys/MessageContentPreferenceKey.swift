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

import SwiftUI

/// `PreferenceKey` for the composer's displayed message content.
public struct MessageContentPreferenceKey: PreferenceKey {
    public static var defaultValue: MessageContent = MessageContent()

    public static func reduce(value: inout MessageContent, nextValue: () -> MessageContent) {
        value = nextValue()
    }
}

@objcMembers
/// Defines message content displayed in the composer.
public class MessageContent: NSObject {
    /// Displayed text, as plain text.
    public let plainText: String
    /// HTML representation of the displayed text.
    public let html: String


    /// Init.
    ///
    /// - Parameters:
    ///   - plainText: Displayed text, as plain text.
    ///   - html: HTML representation of the displayed text.
    public init(plainText: String = "", html: String = "") {
        self.plainText = plainText
        self.html = html
    }
}
