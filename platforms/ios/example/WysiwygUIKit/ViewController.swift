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
import WysiwygComposer
import OSLog

private enum Constants {
    static let maxHeight: CGFloat = 200
}

/// Example UIKit view that adds a WysiwygComposer as well as a button that
/// displays the message (+ HTML representation) the composer would send.
final class ViewController: UIViewController {
    @IBOutlet private weak var wysiwygHostingView: WysiwygHostingView!
    @IBOutlet private weak var sendButton: UIButton!
    @IBOutlet private weak var contentLabel: UILabel!
    @IBOutlet private weak var htmlContentLabel: UILabel!
    @IBOutlet private weak var wysiwygHostingViewHeightConstraint: NSLayoutConstraint!

    override func viewDidLoad() {
        super.viewDidLoad()
        wysiwygHostingView.delegate = self
        sendButton.accessibilityIdentifier = "WysiwygSendButton"
        contentLabel.accessibilityIdentifier = "WysiwygContentLabel"
        htmlContentLabel.accessibilityIdentifier = "WysiwygHtmlContentLabel"
    }
}

private extension ViewController {
    @IBAction func sendButtonTouchedUpInside(_ sender: UIButton) {
        // Get the current content of the composer.
        contentLabel.text = wysiwygHostingView.content.plainText
        htmlContentLabel.text = wysiwygHostingView.content.html
        wysiwygHostingView.clearContent()
    }
}

// Implement delegate that gets notified on composer updates.
extension ViewController: WysiwygHostingViewDelegate {
    func requiredHeightDidChange(_ height: CGFloat) {
        wysiwygHostingViewHeightConstraint.constant = min(Constants.maxHeight, height)
    }

    func isContentEmptyDidChange(_ isEmpty: Bool) {
        sendButton.isEnabled = !isEmpty
    }
}
