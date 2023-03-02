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

enum Users: Identifiable, CaseIterable {
    case alice
    case bob
    case charlie

    var id: String {
        url
    }

    var iconSystemName: String {
        "person.circle"
    }

    var name: String {
        switch self {
        case .alice:
            return "Alice"
        case .bob:
            return "Bob"
        case .charlie:
            return "Charlie"
        }
    }

    var url: String {
        switch self {
        case .alice:
            return "https://matrix.to/#/@alice:matrix.org"
        case .bob:
            return "https://matrix.to/#/@bob:matrix.org"
        case .charlie:
            return "https://matrix.to/#/@charlie:matrix.org"
        }
    }

    var accessibilityIdentifier: WysiwygSharedAccessibilityIdentifier {
        switch self {
        case .alice:
            return .aliceButton
        case .bob:
            return .bobButton
        case .charlie:
            return .charlieButton
        }
    }

    static let title = "Users"

    static func filtered(with text: String) -> [Users] {
        allCases.filter { $0.name.lowercased().contains(text.lowercased()) }
    }
}
