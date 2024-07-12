//
// Copyright 2024 The Matrix.org Foundation C.I.C
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

final class StringLatinLangugesTests: XCTestCase {
    func testLatinLangugeCharacters() throws {
        XCTAssertTrue("hello".containsLatinAndCommonCharactersOnly)
        XCTAssertTrue("helló".containsLatinAndCommonCharactersOnly)
        XCTAssertTrue("helló, ".containsLatinAndCommonCharactersOnly)
        XCTAssertTrue("helló, ".containsLatinAndCommonCharactersOnly)
        XCTAssertTrue("😄🛴🤯❤️".containsLatinAndCommonCharactersOnly)
        // Test the object replacement character as defined in String+Character extension.
        XCTAssertTrue(String.object.containsLatinAndCommonCharactersOnly)
        XCTAssertTrue("!@££$%^&*()".containsLatinAndCommonCharactersOnly)
        
        XCTAssertFalse("你好".containsLatinAndCommonCharactersOnly)
        XCTAssertFalse("感^".containsLatinAndCommonCharactersOnly)
        XCTAssertFalse("Меня зовут Маша".containsLatinAndCommonCharactersOnly)
        XCTAssertFalse("ฉันชอบกินข้าวผัด แต่เธอชอบกินผัดไทย".containsLatinAndCommonCharactersOnly)
        XCTAssertFalse("ni3好^".containsLatinAndCommonCharactersOnly)
    }
}
