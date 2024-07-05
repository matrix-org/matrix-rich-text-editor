/*
Copyright 2022 The Matrix.org Foundation C.I.C.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

/// <reference types="cypress" />

import Chainable = Cypress.Chainable;

declare global {
    // eslint-disable-next-line @typescript-eslint/no-namespace
    namespace Cypress {
        interface Chainable {
            setSelection: typeof setSelection;
            getSelection: typeof getSelection;
        }
    }
}

const getSelection = (): Chainable<string> => {
    // eslint-disable-next-line max-len
    // From https://github.com/cypress-io/cypress/issues/2752#issuecomment-759746305
    return cy.window().then((win) => win.navigator.clipboard.readText());
};

const setSelection = (
    selector: string,
    start: number,
    end: number,
): Chainable<string> => {
    // eslint-disable-next-line max-len
    // From https://github.com/cypress-io/cypress/issues/2839#issuecomment-447012818
    cy.get(selector)
        .trigger('mousedown')
        .then(($el) => {
            const editorEl = $el[0];
            const document = editorEl.ownerDocument;
            const textNode = editorEl.firstChild;
            document
                .getSelection()
                .setBaseAndExtent(textNode, start, textNode, end);
        })
        .trigger('mouseup');
    return cy.document().trigger('selectionchange');
};

Cypress.Commands.add('setSelection', setSelection);
Cypress.Commands.add('getSelection', getSelection);

// Make this file a module
export {};
