/*
Copyright 2023 The Matrix.org Foundation C.I.C.

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

describe('Mention', () => {
    const editor = '.editor:not([disabled])[contenteditable="true"]';
    const mentionText = 'Alice';
    const nbsp = '\u00A0';

    it(
        'insert mention and edit/backspace after mention',
        { browser: 'electron' },
        () => {
            cy.visit('/');
            cy.get(editor).wait(500);
            cy.get(editor).type('@a');
            cy.contains(editor, '@a');

            // click on the @mention button, check mention appears
            cy.contains('Add @mention').click();
            cy.contains(editor, mentionText);

            // type a letter, make sure it's after the mention
            cy.get(editor).type('0');
            cy.contains(editor, mentionText + ' 0');

            // move the cursor left two, to immediately after the link type more
            cy.get(editor).type('{leftArrow}{leftArrow}1');
            cy.contains(editor, mentionText + '1 0');

            // backspace to remove a single letter
            cy.get(editor).type('{Backspace}');
            cy.contains(editor, mentionText + ' 0');

            // backspace to remove the mention
            cy.get(editor).type('{Backspace}');
            cy.contains(editor, ' 0');
        },
    );

    it('edit/backspace before mention', { browser: 'electron' }, () => {
        cy.visit('/');
        cy.get(editor).wait(500);
        cy.get(editor).type('@a');
        cy.contains(editor, '@a');

        // click on the @mention button, check mention appears
        cy.contains('Add @mention').click();
        cy.contains(editor, mentionText);

        // move cursor to before the mention
        cy.get(editor).type('{leftArrow}{leftArrow}');

        // press backspace, make sure nothing happens
        cy.get(editor).type('{Backspace}');
        cy.contains(editor, mentionText);

        // type a letter, make sure it's before the mention
        cy.get(editor).type('0');
        cy.contains(editor, '0' + mentionText);
    });

    it('delete before mention', { browser: 'electron' }, () => {
        cy.visit('/');
        cy.get(editor).wait(500);
        cy.get(editor).type('@a');
        cy.contains(editor, '@a');

        // click on the @mention button, check mention appears
        cy.contains('Add @mention').click();
        cy.contains(editor, mentionText);

        // move cursor to before the mention and press delete, check mention
        // is removed but trailing nbsp remains
        cy.get(editor).type('{leftArrow}{leftArrow}{del}');
        cy.get(editor).should('have.text', nbsp);
    });

    it.skip('delete after mention', { browser: 'electron' }, () => {
        cy.visit('/');
        cy.get(editor).wait(500);
        cy.get(editor).type('@a');
        cy.contains(editor, '@a');

        // click on the @mention button, check mention appears
        cy.contains('Add @mention').click();
        cy.contains(editor, mentionText);

        // add some following text
        cy.get(editor).type('z');
        cy.contains(editor, mentionText + ' z');

        // move cursor to immediately after the mention and press delete,
        // check mention remains with the text following
        cy.get(editor).type('{leftArrow}{leftArrow}{del}');
        cy.contains(editor, mentionText + 'z');
    });

    it.skip('multiple mentions', { browser: 'electron' }, () => {
        cy.visit('/');
        cy.get(editor).wait(500);

        // loop through and put in three mentions
        for (let i = 0; i < 3; i++) {
            cy.get(editor).type('@a');

            // click on the @mention button, check mention appears
            cy.contains('Add @mention').click();
            cy.contains(editor, mentionText);
        }

        // expect we can see three mentions, space separated
        cy.contains(editor, `${mentionText} `.repeat(3));

        // press backspace once, make sure only one mention removed
        cy.get(editor).type('{Backspace}{Backspace}');
        cy.contains(editor, `${mentionText} `.repeat(2));

        // go to the beginning of the editor then press delete
        cy.get(editor).type(`{leftArrow}`.repeat(4));
        cy.get(editor).type('{del}');
        cy.contains(editor, ` ${mentionText} `);
    });
});
