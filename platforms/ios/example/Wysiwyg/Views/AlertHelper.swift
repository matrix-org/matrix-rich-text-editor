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

import SwiftUI
import UIKit

struct AlertHelper<Content: View>: UIViewControllerRepresentable {
    @Binding var isPresented: Bool
    let alert: AlertConfig
    let content: Content

    func makeUIViewController(context _: UIViewControllerRepresentableContext<AlertHelper>) -> UIHostingController<Content> {
        UIHostingController(rootView: content)
    }

    final class Coordinator {
        var alertController: UIAlertController?
        init(_ controller: UIAlertController? = nil) {
            alertController = controller
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func updateUIViewController(_ uiViewController: UIHostingController<Content>,
                                context: UIViewControllerRepresentableContext<AlertHelper>) {
        uiViewController.rootView = content
        var alert = alert
        alert.dismissAction = {
            self.isPresented = false
        }
        if isPresented, uiViewController.presentedViewController == nil {
            context.coordinator.alertController = UIAlertController(alert: alert)
            guard let controller = context.coordinator.alertController else { return }
            uiViewController.present(controller, animated: true)
        }
        if !isPresented, uiViewController.presentedViewController == context.coordinator.alertController {
            uiViewController.dismiss(animated: true)
        }
    }
}

public struct AlertConfig {
    public struct TextFieldData {
        let placeholder: String
        let defaultValue: String?
    }
    
    public enum Action {
        case cancel(title: String)
        case destructive(title: String, action: () -> Void)
        case textAction(title: String, textFieldsData: [TextFieldData], action: ([String]) -> Void)
    }
    
    public var title: String
    public var actions: [Action]
    public var dismissAction: (() -> Void)?
}
