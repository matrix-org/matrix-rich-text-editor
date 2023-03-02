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
import HTMLParser
import UIKit

final class WysiwygPermalinkReplacer: PermalinkReplacer {
    func replacementForLink(_ link: String, text: String) -> NSAttributedString? {
        if #available(iOS 15.0, *),
           link.starts(with: "https://matrix.to/#/"),
           let attachment = WysiwygTextAttachment(displayName: text,
                                                  font: UIFont.preferredFont(forTextStyle: .body)) {
            return NSAttributedString(attachment: attachment)
        } else {
            return nil
        }
    }
}
