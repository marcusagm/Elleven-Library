/**
 * DatePicker component
 *
 * @module DatePicker
 * @description
 * The DatePicker component is a visual calendar selection interface for picking dates.
 * It provides a calendar view for selecting dates and its associated types.
 *
 * @example
 * ```tsx
 * import { DatePicker } from '@/components/ui';
 *
 * <DatePicker
 *   value={new Date()}
 *   onChange={(value) => console.log(value)}
 *   minDate={new Date()}
 *   maxDate={new Date()}
 *   class="custom-class"
 * />
 * ```
 */
export * from './Root';
export * from './types';
