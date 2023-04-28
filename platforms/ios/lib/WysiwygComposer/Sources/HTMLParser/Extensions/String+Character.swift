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

public extension String {
    /// String containing a single NBSP character (`\u{00A0}`)
    static let nbsp = "\u{00A0}"
    /// String containing a single ZWSP character (`\u{200B}`)
    static let zwsp = "\u{200B}"
    /// String containing a single line separator character (`\u{2028}`)
    static let lineSeparator = "\u{2028}"
    /// String containing a single carriage return character (`\r`)
    static let carriageReturn = "\r"
    /// String containing a single line feed character (`\n`)
    static let lineFeed = "\n"
    /// String containing a single slash character(`/`)
    static let slash = "/"
}

public extension Character {
    static let nbsp = Character(.nbsp)
    static let zwsp = Character(.zwsp)
    static let lineFeed = Character(.lineFeed)
    static let slash = Character(.slash)
}
