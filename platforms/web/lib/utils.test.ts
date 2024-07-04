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

import { afterAll, beforeAll } from 'vitest';

import { getUserOperatingSystem } from './utils';

export const mockUserAgent = (ua: string): void => {
    Object.defineProperty(window.navigator, 'userAgent', {
        value: ua,
        writable: true,
    });
};

export const WINDOWS_UA =
    // eslint-disable-next-line max-len
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/42.0.2311.135 Safari/537.36 Edge/12.246';

export const MAC_OS_UA =
    // eslint-disable-next-line max-len
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_2) AppleWebKit/601.3.9 (KHTML, like Gecko) Version/9.0.2 Safari/601.3.9';

export const LINUX_UA =
    // eslint-disable-next-line max-len
    'Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:15.0) Gecko/20100101 Firefox/15.0.1';

export const IOS_UA =
    // eslint-disable-next-line max-len
    'Mozilla/5.0 (iPhone; CPU iPhone OS 12_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/69.0.3497.105 Mobile/15E148 Safari/605.1';

export const ANDROID_UA =
    // eslint-disable-next-line max-len
    'Mozilla/5.0 (Linux; Android 13; SM-S901B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Mobile Safari/537.36';

describe('utils', () => {
    describe('getUserOperatingSystem', () => {
        let originalUserAgent = '';

        beforeAll(() => {
            originalUserAgent = navigator.userAgent;
        });

        afterAll(() => {
            mockUserAgent(originalUserAgent);
        });

        test('returns null for unknown operating systems', () => {
            mockUserAgent('wut?!');
            const os = getUserOperatingSystem();
            expect(os).toBeNull();
        });

        test.each([
            ['Windows', WINDOWS_UA],
            ['macOS', MAC_OS_UA],
            ['Linux', LINUX_UA],
            ['iOS', IOS_UA],
            ['Android', ANDROID_UA],
        ])(
            'should correctly detect %s',
            (expectedOperatingSystem, userAgent) => {
                mockUserAgent(userAgent);
                const os = getUserOperatingSystem();
                expect(os).toBe(expectedOperatingSystem);
            },
        );
    });
});
