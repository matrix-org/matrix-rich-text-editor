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
import Combine
import OSLog

/// Declares methods that should be adopted by an object that aim to react on the Wysiwyg composer actions.
@objc public protocol WysiwygHostingViewDelegate: AnyObject {
    /// Tells the delegate that the current idealHeight of the composer has updated.
    ///
    /// - Parameters:
    ///   - height: new required height
    func idealHeightDidChange(_ height: CGFloat)
    /// Tells the delegate that the composer empty content state has updated.
    ///
    /// - Parameters:
    ///   - isEmpty: whether the composer is empty or not.
    func isContentEmptyDidChange(_ isEmpty: Bool)
}

/// Hosting view that provides support for Wysiwyg UIKit implementation.
@objcMembers
public final class WysiwygHostingView: UIView {
    // MARK: - Public
    /// The delegate of the `WysiwygHostingView`.
    public weak var delegate: WysiwygHostingViewDelegate?
    /// The content currently displayed in the composer.
    public var content: WysiwygComposerContent {
        return viewModel.content
    }

    // MARK: - Private
    @ObservedObject private var viewModel = WysiwygComposerViewModel()
    private var cancellables: Set<AnyCancellable>?

    // MARK: - Public
    /// Apply given action to the composer.
    ///
    /// - Parameters:
    ///   - action: Action to apply.
    public func apply(_ action: WysiwygAction) {
        viewModel.apply(action)
    }

    /// Clear the content of the composer.
    public func clearContent() {
        viewModel.clearContent()
    }

    // MARK: - Override
    public override func awakeFromNib() {
        super.awakeFromNib()

        let wysiwygView = WysiwygView()
            .environmentObject(viewModel)

        // Subscribe to relevant events and map them to UIKit-style delegate.
        cancellables = [
            viewModel.$isContentEmpty
                .removeDuplicates()
                .sink(receiveValue: { [unowned self] isContentEmpty in
                    self.delegate?.isContentEmptyDidChange(isContentEmpty)
                }),
            viewModel.$idealHeight
                .removeDuplicates()
                .sink(receiveValue: { [unowned self] idealHeight in
                    self.delegate?.idealHeightDidChange(idealHeight)
                }),
        ]

        // Attach the view to a hosting controller and display it's UIView container.
        let hostingController = UIHostingController(rootView: wysiwygView)
        addSubViewMatchingParent(hostingController.view)
    }
}
