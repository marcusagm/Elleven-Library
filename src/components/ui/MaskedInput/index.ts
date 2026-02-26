/**
 * MaskedInput Component
 *
 * @module MaskedInput
 * @description
 * The MaskedInput component is a specialized Input component that applies a format mask to the user input.
 * It supports '0' as a placeholder for numeric digits.
 *
 * @example
 * ```tsx
 * import { MaskedInput } from '@/components/ui';
 *
 * <MaskedInput
 *   mask="00/00/0000"
 *   placeholder="DD/MM/YYYY"
 *   onInput={() => {}}
 *   value="10/10/2020"
 *   class="custom-class"
 *   wrapperClass="custom-wrapper-class"
 *   label="custom-label"
 *   error={false}
 *   errorMessage="custom-error-message"
 *   size="md"
 * />
 * ```
 */
export * from './MaskedInput';
export * from './types';
