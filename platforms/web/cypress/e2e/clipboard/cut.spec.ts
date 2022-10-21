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

describe('Cut', () => {
    const editor = '.editor:not([disabled])[contenteditable="true"]';

    it('remove text that is cut to clipboard', { browser: 'electron' }, () => {
        cy.visit('/');
        cy.get(editor).wait(500);
        cy.get(editor).type('firstREMOVEME');
        cy.contains(editor, 'firstREMOVEME');

        cy.setSelection(editor, 5, 13);
        cy.document().invoke('execCommand', 'cut');

        // The clipboard should contain the text we cut
        cy.getSelection().should('equal', 'REMOVEME');

        // Clear the selection, so we really check that the cut is working,
        // instead of typing over the selected text
        cy.setSelection(editor, 5, 5);

        // Type something, because only when we do that do we reveal what was
        // really in the Rust model.
        cy.get(editor).type('last');
        cy.contains(editor, 'last');
        cy.get(editor).should('not.contain', 'REMOVEME');
    });
});
