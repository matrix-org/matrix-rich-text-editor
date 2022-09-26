import { SelectTuple, Tuple } from './types';

export function isSelectTuple(tuple: Tuple): tuple is SelectTuple {
    return tuple[0] === 'select';
}
