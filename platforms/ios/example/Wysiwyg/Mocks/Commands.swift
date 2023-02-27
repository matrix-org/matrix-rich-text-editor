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

enum Commands: Identifiable, CaseIterable {
    case join
    case invite
    case me

    var id: String {
        name
    }

    var iconSystemName: String {
        "terminal"
    }

    var name: String {
        switch self {
        case .join:
            return "/join"
        case .invite:
            return "/invite"
        case .me:
            return "/me"
        }
    }

    static let title = "Commands"

    static func filtered(with text: String) -> [Commands] {
        allCases.filter { $0.name.lowercased().contains("/" + text.lowercased()) }
    }
}
