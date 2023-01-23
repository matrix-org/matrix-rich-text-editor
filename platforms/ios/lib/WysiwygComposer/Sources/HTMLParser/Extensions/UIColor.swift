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

public extension UIColor {
    func toHexString(shouldIncludeAlpha: Bool = false) -> String {
        var r: CGFloat = 0
        var g: CGFloat = 0
        var b: CGFloat = 0
        var a: CGFloat = 0
        getRed(&r, green: &g, blue: &b, alpha: &a)
        let hexValue: Int
        if shouldIncludeAlpha {
            hexValue = Int(r * 255) << 24 | Int(g * 255) << 16 | Int(b * 255) << 8 | Int(a * 255) << 0
            return String(format: "#%08x", hexValue)
        } else {
            hexValue = Int(r * 255) << 16 | Int(g * 255) << 8 | Int(b * 255) << 0
            return String(format: "#%06x", hexValue)
        }
    }
}
