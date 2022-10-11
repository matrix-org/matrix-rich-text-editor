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

describe('Paste', () => {
    const editor = '.editor:not([disabled])[contenteditable="true"]';

    it(
        'should display pasted text after we type',
        { browser: 'electron' },
        () => {
            cy.visit('/');
            cy.get(editor).type('BEFORE');
            cy.window()
                .its('navigator.clipboard')
                .invoke('writeText', 'pasted');
            cy.get(editor).focus();
            cy.document().invoke('execCommand', 'paste');
            cy.get(editor).type('AFTER');
            cy.get(editor).contains(/^BEFOREpastedAFTER/);
        },
    );

    it(
        'should convert pasted newlines into BRs',
        { browser: 'electron' },
        () => {
            cy.visit('/');
            cy.window()
                .its('navigator.clipboard')
                .invoke('writeText', 'aa\nbb');
            cy.get(editor).focus();
            cy.document().invoke('execCommand', 'paste');
            cy.get(editor)
                .invoke('html')
                .then((html) => expect(html).to.equal('aa<br>bb<br>'));
            // (Note the extra BR is always added at the end)
        },
    );
});
