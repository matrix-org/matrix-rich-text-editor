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

/**
 * Findss the operating system of the user
 * @returns {string|null} the operating system, `null` if the operating system is unknown
 */
export function getUserOperatingSystem():
    | 'Windows'
    | 'macOS'
    | 'Linux'
    | 'iOS'
    | 'Android'
    | null {
    const userAgent = navigator.userAgent.toLowerCase();
    if (userAgent.includes('iphone') || userAgent.includes('ipad')) {
        return 'iOS';
    } else if (userAgent.includes('android')) {
        return 'Android';
    } else if (userAgent.includes('win')) {
        return 'Windows';
    } else if (userAgent.includes('mac')) {
        return 'macOS';
    } else if (userAgent.includes('linux')) {
        return 'Linux';
    } else {
        return null;
    }
}
