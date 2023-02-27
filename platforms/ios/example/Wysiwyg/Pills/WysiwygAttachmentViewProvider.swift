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
import WysiwygComposer

/// Provider for mention Pills attachment view.
@available(iOS 15.0, *)
@objc class WysiwygAttachmentViewProvider: NSTextAttachmentViewProvider {
    // MARK: - Properties

    static let pillUTType = "org.matrix.rte.pills"

    private static let pillAttachmentViewSizes = WysiwygAttachmentView.Sizes(verticalMargin: 2.0,
                                                                             horizontalMargin: 4.0,
                                                                             avatarSideLength: 16.0)
    private weak var textView: UITextView?

    // MARK: - Override

    override init(textAttachment: NSTextAttachment,
                  parentView: UIView?,
                  textLayoutManager: NSTextLayoutManager?,
                  location: NSTextLocation) {
        super.init(textAttachment: textAttachment, parentView: parentView, textLayoutManager: textLayoutManager, location: location)

        textView = parentView?.superview as? UITextView
    }

    override func loadView() {
        super.loadView()

        guard let textAttachment = textAttachment as? WysiwygTextAttachment else {
            return
        }

        guard let pillData = textAttachment.data else {
            return
        }

        let pillView = WysiwygAttachmentView(frame: CGRect(origin: .zero, size: Self.size(forDisplayText: pillData.displayName,
                                                                                          andFont: pillData.font)),
                                             sizes: Self.pillAttachmentViewSizes,
                                             andPillData: pillData)
        view = pillView
        WysiwygPillsFlusher.registerPillView(pillView)
    }
}

@available(iOS 15.0, *)
extension WysiwygAttachmentViewProvider {
    /// Computes size required to display a pill for given display text.
    ///
    /// - Parameters:
    ///   - displayText: display text for the pill
    ///   - font: the text font
    /// - Returns: required size for pill
    static func size(forDisplayText displayText: String, andFont font: UIFont) -> CGSize {
        let label = UILabel(frame: .zero)
        label.text = displayText
        label.font = font
        let labelSize = label.sizeThatFits(CGSize(width: CGFloat.greatestFiniteMagnitude,
                                                  height: pillAttachmentViewSizes.pillBackgroundHeight))

        return CGSize(width: labelSize.width + pillAttachmentViewSizes.totalWidthWithoutLabel,
                      height: pillAttachmentViewSizes.pillHeight)
    }
}
