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

extension UITextView {
    /// Apply given content to the text view. This will temporary disrupt the text view
    /// delegate in order to avoid having multiple unnecessary selection frowarded to
    /// the model. This is especially useful since setting the attributed text automatically
    /// moves the cursor to the end of the text and it might not be the expected behaviour.
    ///
    /// - Parameters:
    ///   - content: Content to apply.
    func apply(_ content: WysiwygComposerAttributedContent) {
        performWithoutDelegate {
            self.attributedText = content.text
            // Set selection to {0, 0} then to expected position
            // avoids an issue with autocapitalization.
            self.selectedRange = .zero
            self.selectedRange = content.selection
        }
    }
}

private extension UITextView {
    /// Perform an action while temporary removing the text view delegate.
    ///
    /// - Parameters:
    ///   - block: Code block to perform.
    func performWithoutDelegate(block: () -> Void) {
        let myDelegate = delegate
        delegate = nil
        block()
        delegate = myDelegate
    }
}
