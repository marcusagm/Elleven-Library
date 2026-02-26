/**
 * @module ProgressBar
 * Components for indicating binary or relative progress of long-running operations.
 *
 * @example
 * ```tsx
 * import { ProgressBar } from '@/components/ui';
 *
 * <ProgressBar
 *     value={50}
 *     maximumValue={100}
 *     size="md"
 *     isLabelVisible={true}
 *     labelTitle="Progress"
 *     isIndeterminate={false}
 *     class="my-class"
 * />
 * ```
 */

export * from './ProgressBar';
export * from './types';
