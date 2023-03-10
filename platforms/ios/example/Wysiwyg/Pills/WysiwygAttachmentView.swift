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

/// Base view class for mention Pills.
@available(iOS 15.0, *)
@objcMembers
class WysiwygAttachmentView: UIView {
    // MARK: - Internal Structs

    /// Sizes provided alongside frame to build `PillAttachmentView` layout.
    struct Sizes {
        var verticalMargin: CGFloat
        var horizontalMargin: CGFloat
        var avatarSideLength: CGFloat

        var pillBackgroundHeight: CGFloat {
            avatarSideLength + 2 * verticalMargin
        }

        var pillHeight: CGFloat {
            pillBackgroundHeight + 2 * verticalMargin
        }

        var displaynameLabelLeading: CGFloat {
            avatarSideLength + 2 * horizontalMargin
        }

        var totalWidthWithoutLabel: CGFloat {
            displaynameLabelLeading + 2 * horizontalMargin
        }
    }

    // MARK: - Init

    /// Create a Mention Pill view for given data.
    ///
    /// - Parameters:
    ///   - frame: the frame of the view
    ///   - sizes: additional size parameters
    ///   - pillData: the pill data
    convenience init(frame: CGRect,
                     sizes: Sizes,
                     andPillData pillData: WysiwygTextAttachmentData) {
        self.init(frame: frame)
        let label = UILabel(frame: .zero)
        label.text = pillData.displayName
        label.font = pillData.font
        label.textColor = UIColor.label
        label.accessibilityIdentifier = "WysiwygAttachmentViewLabel" + pillData.displayName
        let labelSize = label.sizeThatFits(CGSize(width: CGFloat.greatestFiniteMagnitude,
                                                  height: sizes.pillBackgroundHeight))
        label.frame = CGRect(x: sizes.displaynameLabelLeading,
                             y: 0,
                             width: labelSize.width,
                             height: sizes.pillBackgroundHeight)

        let pillBackgroundView = UIView(frame: CGRect(x: 0,
                                                      y: sizes.verticalMargin,
                                                      width: labelSize.width + sizes.totalWidthWithoutLabel,
                                                      height: sizes.pillBackgroundHeight))

        let avatarView = UIImageView(frame: CGRect(x: sizes.horizontalMargin,
                                                   y: sizes.verticalMargin,
                                                   width: sizes.avatarSideLength,
                                                   height: sizes.avatarSideLength))
        avatarView.image = UIImage(systemName: "person.circle")?.withRenderingMode(.alwaysTemplate)
        avatarView.tintColor = UIColor.label

        avatarView.isUserInteractionEnabled = false

        pillBackgroundView.addSubview(avatarView)
        pillBackgroundView.addSubview(label)

        pillBackgroundView.backgroundColor = UIColor(red: 227 / 255, green: 232 / 255, blue: 240 / 255, alpha: 1.0)
        pillBackgroundView.layer.cornerRadius = sizes.pillBackgroundHeight / 2.0

        addSubview(pillBackgroundView)
    }

    // MARK: - Override

    override var isHidden: Bool {
        get {
            false
        }
        // swiftlint:disable:next unused_setter_value
        set {
            // Disable isHidden for pills, fixes a bug where the system sometimes
            // hides attachment views for undisclosed reasons. Pills never needs to be hidden.
        }
    }
}
