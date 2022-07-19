//
//  ContentView.swift
//  Wysiwyg
//
//  Created by Arnaud Ringenbach on 19/07/2022.
//

import SwiftUI
import WysiwygComposer


struct ContentView: View {
    var body: some View {
        WysiwygTextView()
            .padding()
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
