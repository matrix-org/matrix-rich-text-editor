import XCTest
@testable import WysiwygComposer

final class WysiwygComposerTests: XCTestCase {
    func testExample() throws {
        let composer = newComposerModel()
        let update = composer.replaceText(newText: "Test")
        switch update.textUpdate() {
        case .keep:
            XCTFail()
        case .replaceAll(replacementHtml: let html,
                         selectionStartCodepoint: _,
                         selectionEndCodepoint: _):
            XCTAssertEqual(html, "Test")
        }
    }
}
