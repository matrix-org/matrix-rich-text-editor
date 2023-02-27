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

enum Rooms: Identifiable, CaseIterable {
    case room1
    case room2
    case room3

    var id: String {
        url
    }

    var iconSystemName: String {
        "character.bubble"
    }

    var name: String {
        switch self {
        case .room1:
            return "Room 1"
        case .room2:
            return "Room 2"
        case .room3:
            return "Room 3"
        }
    }

    var url: String {
        switch self {
        case .room1:
            return "https://matrix.to/#/#room1:matrix.org"
        case .room2:
            return "https://matrix.to/#/#room2:matrix.org"
        case .room3:
            return "https://matrix.to/#/#room3:matrix.org"
        }
    }

    static let title = "Rooms"

    static func filtered(with text: String) -> [Rooms] {
        allCases.filter { $0.name.lowercased().contains(text.lowercased()) }
    }
}
