//
//  WysiwygView.swift
//  
//
//  Created by Arnaud Ringenbach on 19/07/2022.
//

import SwiftUI

public struct WysiwygView: View {
    // MARK: - Public
    public var body: some View {
        VStack {
            WysiwygComposerView(viewState: viewModel.viewState,
                                change: viewModel.didAttemptChange,
                                textDidUpdate: viewModel.textDidUpdate,
                                textDidChangeSelection: viewModel.textDidChangeSelection)
            Button("Bold") {
                viewModel.applyBold()
            }.buttonStyle(.automatic)
        }

    }

    public init() {}

    // MARK: - Internal
    @StateObject var viewModel = WysiwygComposerViewModel()
}
