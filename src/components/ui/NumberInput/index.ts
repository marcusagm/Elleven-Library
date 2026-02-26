/**
 * NumberInput component
 *
 * @module NumberInput
 * @description
 * The NumberInput component is a specialized Input component that allows the user to enter numeric values.
 * It provides controls for incrementing/decrementing the value and enforces numeric validation.
 *
 * @example
 * ```tsx
 * import { NumberInput } from '@/components/ui';
 *
 * <NumberInput
 *   min={0}
 *   max={100}
 *   step={1}
 *   value={0}
 *   onChange={handleChange}
 *   disabled={false}
 *   class="number-input"
 * />
 * ```
 */
export * from './NumberInput';
export * from './types';
