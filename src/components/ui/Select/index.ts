/**
 * @module Select
 * A comprehensive selection component suite.
 * Supports a High-level Select component and a Compound Component pattern for maximum flexibility.
 *
 * @example
 * ```tsx
 * import { Select } from './Select';
 *
 * <Select options={[{ value: '1', label: 'Option 1' }]} />
 * ```
 */

import { Root } from './Root';
import { Trigger } from './Trigger';
import { Value } from './Value';
import { Content } from './Content';
import { Item } from './Item';
import { Search } from './Search';

export * from './Select';
export * from './types';

/**
 * Compound component parts for building custom Select implementations.
 */
export const SelectPrimitive = {
    Root,
    Trigger,
    Value,
    Content,
    Item,
    Search
};
