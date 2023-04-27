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

extension UITextView {
    /// Toggles  autocorrection if needed. It should always be enabled,
    /// unless current text starts with exactly one slash.
    func toggleAutocorrectionIfNeeded() {
        let newValue: UITextAutocorrectionType = attributedText.string.prefix(while: { $0 == .slash }).count == 1 ? .no : .yes
        if newValue != autocorrectionType {
            autocorrectionType = newValue
            reloadInputViews()
        }
    }
}
