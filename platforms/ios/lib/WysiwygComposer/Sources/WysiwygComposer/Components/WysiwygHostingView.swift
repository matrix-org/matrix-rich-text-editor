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
import SwiftUI
import OSLog

/// Declares methods that should be adopted by an object that aim to react on the Wysiwyg composer actions.
@objc public protocol WysiwygHostingViewDelegate: AnyObject {
    /// Tells the delegate that the current requiredHeight of the composer has updated.
    ///
    /// - Parameters:
    ///   - height: new required height
    func requiredHeightDidChange(_ height: CGFloat)
    /// Tells the delegate that the composer empty content state has updated.
    ///
    /// - Parameters:
    ///   - isEmpty: whether the composer is empty or not.
    func isEmptyContentDidChange(_ isEmpty: Bool)
}

/// Hosting view that provides support for Wysiwyg UIKit implementation.
@objcMembers
public final class WysiwygHostingView: UIView {
    // MARK: - Public
    /// The delegate of the `WysiwygHostingView`.
    public weak var delegate: WysiwygHostingViewDelegate?
    /// The content currently displayed in the composer.
    public private(set) var content: MessageContent = MessageContent()

    // MARK: - Override
    public override func awakeFromNib() {
        super.awakeFromNib()

        // Create the SwiftUI view and map its preference keys changes to delegate and properties.
        let wysiwygView = WysiwygView()
            .onPreferenceChange(MessageContentPreferenceKey.self) { [unowned self] (messageContent: MessageContent) in
                self.content = messageContent
            }
            .onPreferenceChange(RequiredHeightPreferenceKey.self) { [unowned self] (height: CGFloat) in
                self.delegate?.requiredHeightDidChange(height)
            }
            .onPreferenceChange(IsEmptyContentPreferenceKey.self) { [unowned self] (isEmpty: Bool) in
                self.delegate?.isEmptyContentDidChange(isEmpty)
            }

        // Attach the view to a hosting controller and display it's UIView container.
        let hostingController = UIHostingController(rootView: wysiwygView)
        addSubViewMatchingParent(hostingController.view)
    }
}
