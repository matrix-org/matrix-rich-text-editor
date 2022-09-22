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

enum TestConstants {
    /// Test string with emojis inputed both with codepoints and Xcode emoji insertion.
    /// String is actually 6 char long "abc🎉🎉👩🏿‍🚀" and represents 14 UTF-16 code units (3+2+2+7)
    static let testStringWithEmojis = "abc🎉\u{1f389}\u{1F469}\u{1F3FF}\u{200D}\u{1F680}"
    static let testStringAfterBackspace = "abc🎉🎉"
}
