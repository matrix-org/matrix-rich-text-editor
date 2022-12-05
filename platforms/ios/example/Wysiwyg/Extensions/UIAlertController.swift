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

extension UIAlertController {
    convenience init(alert: AlertConfig) {
        self.init(title: alert.title, message: nil, preferredStyle: .alert)
        var numberOfTextFields = 0
        for action in alert.actions {
            switch action {
            case let .cancel(title):
                addAction(UIAlertAction(title: title, style: .cancel) { _ in
                    alert.dismissAction?()
                })
            case let .destructive(title, action):
                addAction(UIAlertAction(title: title, style: .destructive) { _ in
                    alert.dismissAction?()
                    action()
                })
            case let .textAction(title: title, textsData, action):
                for textData in textsData {
                    addTextField()
                    textFields![numberOfTextFields].placeholder = textData.placeholder
                    textFields![numberOfTextFields].text = textData.defaultValue
                    numberOfTextFields += 1
                }
                let textFields = textFields!
                addAction(UIAlertAction(title: title, style: .default) { _ in
                    alert.dismissAction?()
                    var strings: [String] = []
                    for textField in textFields {
                        strings.append(textField.text ?? "")
                    }
                    action(strings)
                })
            }
        }
    }
}
