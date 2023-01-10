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

@testable import WysiwygComposer
import XCTest

final class NSRangeTests: XCTestCase {
    func testCreateRangeInBetween() {
        XCTAssertEqual(
            NSRange(between: .init(location: 0, length: 3), and: .init(location: 6, length: 12)),
            NSRange(location: 3, length: 3)
        )
        XCTAssertEqual(
            NSRange(between: .init(location: 3, length: 3), and: .init(location: 6, length: 12)),
            NSRange(location: 6, length: 0)
        )
        XCTAssertNil(NSRange(between: .init(location: 6, length: 12), and: .init(location: 3, length: 3)))
    }
}
