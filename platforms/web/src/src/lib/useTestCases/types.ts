import { useTestCases } from './useTestCases';

export type TestUtilities = ReturnType<typeof useTestCases>['utilities'];

export type SelectTuple = ['select', number, number];
export type Tuple = SelectTuple | [string, (string | number)?, (string | number)?];
export type Actions = Array<Tuple>;
