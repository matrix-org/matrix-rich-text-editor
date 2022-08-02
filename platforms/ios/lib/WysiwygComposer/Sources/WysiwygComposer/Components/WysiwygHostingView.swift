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

@objc public protocol WysiwygHostingViewDelegate: AnyObject {
    func requiredHeightDidChange(_ height: CGFloat)
    func isEmptyContentDidChange(_ isEmpty: Bool)
}

@objcMembers
public final class WysiwygHostingView: UIView {
    public weak var delegate: WysiwygHostingViewDelegate?

    public private(set) var content: MessageContent = MessageContent()

    public override func awakeFromNib() {
        super.awakeFromNib()
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

        let hostingController = UIHostingController(rootView: wysiwygView)
        addSubViewMatchingParent(hostingController.view)
    }
}
