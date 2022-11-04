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

import OSLog
import UIKit

// MARK: - Logger

extension Logger {
    // MARK: Internal

    static var subsystem = "org.matrix.WysiwygComposer"

    /// Creates a customized log for debug.
    ///
    /// - Parameters:
    ///   - elements: Elements to log.
    ///   - functionName: Name from the function where it is called.
    func logDebug(_ elements: [String], functionName: String) {
        debug("\(customLog(elements, functionName: functionName))")
    }

    /// Creates a customized error log.
    ///
    /// - Parameters:
    ///   - elements: Elements to log.
    ///   - functionName: Name from the function where it is called.
    func logError(_ elements: [String], functionName: String) {
        error("\(customLog(elements, functionName: functionName))")
    }

    /// Creates a customized warning log.
    ///
    /// - Parameters:
    ///   - elements: Elements to log.
    ///   - functionName: Name from the function where it is called.
    func logWarning(_ elements: [String], functionName: String) {
        warning("\(customLog(elements, functionName: functionName))")
    }

    // MARK: Private

    private func customLog(_ elements: [String], functionName: String) -> String {
        var logMessage = elements.map { $0 + " | " }.joined()
        logMessage.append(contentsOf: functionName)
        return logMessage
    }
}

// MARK: - UITextView + Logger

extension UITextView {
    /// Returns a log ready description of the current selection.
    var logSelection: String {
        "Sel(att): \(selectedRange)"
    }

    /// Returns a log ready description of the current text..
    var logText: String {
        "Text: \"\(text ?? "")\""
    }
}

// MARK: - WysiwygComposerAttributedContent + Logger

extension WysiwygComposerAttributedContent {
    /// Returns a log ready description of the attributed selection.
    var logSelection: String {
        "Sel(att): \(selection)"
    }

    /// Returns a log ready description of the markdown text.
    var logText: String {
        "Text: \"\(text)\""
    }
}
