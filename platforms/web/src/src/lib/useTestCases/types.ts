import { useTestCases } from './useTestCases';

export type TestUtilities = ReturnType<typeof useTestCases>['utilities'];
export type Actions = Array<[string, any?, any?]>;
