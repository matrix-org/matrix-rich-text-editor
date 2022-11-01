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

public class PlaceholdableTextView: UITextView {
    var shouldShowPlaceholder = true {
        didSet {
            setNeedsDisplay()
        }
    }
    
    var placeholder: String? {
        didSet {
            setNeedsDisplay()
        }
    }
    
    var placeholderColor: UIColor = .placeholderText {
        didSet {
            setNeedsDisplay()
        }
    }
    
    var placeholderFont = UIFont.preferredFont(forTextStyle: .subheadline) {
        didSet {
            setNeedsDisplay()
        }
    }
    
    override public init(frame: CGRect, textContainer: NSTextContainer?) {
        super.init(frame: frame, textContainer: textContainer)
        contentMode = .redraw
    }
    
    required init?(coder: NSCoder) {
        super.init(coder: coder)
        contentMode = .redraw
    }
    
    override public func draw(_ rect: CGRect) {
        super.draw(rect)
        
        guard shouldShowPlaceholder, let placeholder = placeholder else {
            return
        }
        
        let attributes: [NSAttributedString.Key: Any] = [.foregroundColor: placeholderColor, .font: placeholderFont]
        
        let frame = rect.inset(by: .init(top: textContainerInset.top,
                                         left: textContainerInset.left + textContainer.lineFragmentPadding,
                                         bottom: textContainerInset.bottom,
                                         right: textContainerInset.right))
        
        placeholder.draw(in: frame, withAttributes: attributes)
    }
}
