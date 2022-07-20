//
//  WysiwygComposerViewModel.swift
//  
//
//  Created by Arnaud Ringenbach on 19/07/2022.
//

import Foundation

struct WysiwygComposerViewState {
    var textSelection: NSRange
    var html: String
}

class WysiwygComposerViewModel: ObservableObject {
    @Published var viewState: WysiwygComposerViewState

    private var operationQueue: OperationQueue
    private var model: ComposerModel

    static let initialText = ""
    init() {
        self.operationQueue = OperationQueue()
        self.operationQueue.maxConcurrentOperationCount = 1
        self.operationQueue.qualityOfService = .userInteractive
        self.model = newComposerModel()
        self.viewState = WysiwygComposerViewState(
            textSelection: .init(location: 0, length: 0),
            html: ""
        )
    }

    func didAttemptChange(of text: String, range: NSRange, replacementText: String) {
        operationQueue.addOperation {
            let update: ComposerUpdate
            if replacementText == "" {
                update = self.model.backspace()
            } else {
                update = self.model.replaceText(newText: replacementText)
            }
            self.applyUpdate(update)
       }
    }

    func textDidUpdate(text: String, range: NSRange) {
        operationQueue.addOperation {
            // TODO if needed
        }
    }

    func textDidChangeSelection(text: String, range: NSRange) {
        operationQueue.addOperation {
            self.model.select(startUtf16Codeunit: UInt32(range.location),
                              endUtf16Codeunit: UInt32(range.location+range.length))

        }
    }

    func applyBold() {
        operationQueue.addOperation {
            let update = self.model.bold()
            self.applyUpdate(update)
        }
    }
}

private extension WysiwygComposerViewModel {
    func applyUpdate(_ update: ComposerUpdate) {
        switch update.textUpdate() {
        case .replaceAll(replacementHtml: let codeUnits,
                         startUtf16Codeunit: let start,
                         endUtf16Codeunit: let end):
            let html = String(utf16CodeUnits: codeUnits, count: codeUnits.count)
            print("WYSIWYG: Current HTML: \(html)")
            print("WYSIWYG: Indexes: \(start) -> \(end)")
            DispatchQueue.main.async {
                self.viewState = WysiwygComposerViewState(
                    textSelection: NSRange(location: Int(start),
                                           length: Int(end-start)),
                    html: html
                )
            }
        default:
            break
        }
    }
}
