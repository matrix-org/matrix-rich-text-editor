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

    init?(tempHexColor: String) {
        switch tempHexColor {
        case Self.inlineCode.tempHexColor:
            self = .inlineCode
        case Self.codeBlock.tempHexColor:
            self = .codeBlock
        case Self.quote.tempHexColor:
            self = .quote
        default:
            return nil
        }
    }

    var tempColor: UIColor {
        switch self {
        case .inlineCode:
            return UIColor.systemBlue
        case .codeBlock:
            return UIColor.systemGreen
        case .quote:
            return UIColor.systemRed
        }
    }

    var tempHexColor: String {
        tempColor.toHexString()
    }

    // TODO: should be configurable
    var backgroundColor: UIColor {
        UIColor(red: 244 / 255,
                green: 246 / 255,
                blue: 250 / 255,
                alpha: 1.0)
    }

    // quinary content
    var borderColor: UIColor {
        UIColor(red: 227 / 255,
                green: 232 / 255,
                blue: 240 / 255,
                alpha: 1.0)
    }

    var borderWidth: CGFloat {
        1.0
    }

    var cornerRadius: CGFloat {
        4.0
    }
}
