/**
 * DateInput component
 *
 * @module DateInput
 * @description
 * The DateInput component is a specialized Input component that allows the user to enter dates.
 * It provides a masked date entry field and its associated types.
 *
 * @example
 * ```tsx
 * import { DateInput } from '@/components/ui';
 *
 * <DateInput
 *   placeholder="DD/MM/YYYY"
 *   onInput={(value) => console.log(value)}
 *   value="10/10/2020"
 *   defaultValue="10/10/2020"
 *   onChange={(value) => console.log(value)}
 *   class="custom-class"
 *   wrapperClass="custom-wrapper-class"
 *   label="custom-label"
 *   error={false}
 *   errorMessage="custom-error-message"
 *   size="md"
 * />
 * ```
 */
export * from './DateInput';
export * from './types';
