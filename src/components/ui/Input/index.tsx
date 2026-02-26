/**
 * Input component
 *
 * @module Input
 * @description
 * The Input component is a standardized text input field for the Mundam UI.
 * It provides a base component for all input types and handles keyboard events.
 *
 * @example
 * ```tsx
 * import { Input } from '@/components/ui';
 *
 * <Input
 *   value="value"
 *   onInput={() => {}}
 *   placeholder="Enter your name"
 *   class="custom-class"
 *   wrapperClass="custom-wrapper-class"
 *   label="custom-label"
 *   error={false}
 *   errorMessage="custom-error-message"
 *   size="md"
 * />
 * ```
 */

export * from './Input';
export * from './types';
