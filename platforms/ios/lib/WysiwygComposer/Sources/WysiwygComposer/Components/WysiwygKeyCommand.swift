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

/// An enum describing key commands that can be handled by the hosting application.
/// This can be done by providing a `KeyCommandHandler` to the `WysiwygComposerView`.
/// If handler is nil, or if the handler returns false, a default behaviour will be applied (see cases description).
public enum WysiwygKeyCommand: CaseIterable {
    /// User pressed `enter`. Default behaviour: a line feed is created.
    /// Note: in the context of a messaging app, this is usually used to send a message.
    case enter
    /// User pressed `shift` + `enter`. Default behaviour: a line feed is created.
    case shiftEnter

    var input: String {
        switch self {
        case .enter, .shiftEnter:
            return "\r"
        }
    }

    var modifierFlags: UIKeyModifierFlags {
        switch self {
        case .enter:
            return []
        case .shiftEnter:
            return .shift
        }
    }

    static func from(_ keyCommand: UIKeyCommand) -> WysiwygKeyCommand? {
        WysiwygKeyCommand.allCases.first(where: { $0.input == keyCommand.input && $0.modifierFlags == keyCommand.modifierFlags })
    }
}
