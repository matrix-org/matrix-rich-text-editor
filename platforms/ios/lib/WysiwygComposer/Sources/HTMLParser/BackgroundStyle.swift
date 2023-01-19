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

internal enum BackgroundStyle {
    case inlineCode
    case codeBlock
    case quote

    var tempColor: UIColor {
        switch self {
        case .inlineCode:
            return UIColor.blue
        case .codeBlock:
            return UIColor.green
        case .quote:
            return UIColor.red
        }
    }

    var tempHexColor: String {
        tempColor.toHexString()
    }

    var backgroundColor: UIColor {
        switch self {
        case .inlineCode, .codeBlock:
            return HTMLParser.style.codeBackgroundColor
        case .quote:
            return HTMLParser.style.quoteBackgroundColor
        }
    }

    var borderColor: UIColor {
        switch self {
        case .inlineCode, .codeBlock:
            return HTMLParser.style.codeBorderColor
        case .quote:
            return HTMLParser.style.quoteBorderColor
        }
    }
}
