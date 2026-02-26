/**
 * Toggle component exports.
 *
 * @module Toggle
 * @description
 * The Toggle component is a specialized component for displaying toggles.
 *
 * @example
 * ```tsx
 * import { Toggle } from '@/components/ui';
 *
 * <Toggle
 *   class="custom-class"
 *   pressed={false}
 *   defaultPressed={false}
 *   onPressedChange={(pressed) => console.log(pressed)}
 *   variant="default"
 *   size="md"
 *   disabled={false}
 * >
 *   Toggle
 * </Toggle>
 * ```
 *
 * @example
 * ```tsx
 * import { ToggleGroup, ToggleGroupItem } from '@/components/ui';
 *
 * <ToggleGroup>
 *   <ToggleGroupItem value="1">
 *     Toggle
 *   </ToggleGroupItem>
 *   <ToggleGroupItem value="2">
 *     Toggle
 *   </ToggleGroupItem>
 *   <ToggleGroupItem value="3">
 *     Toggle
 *   </ToggleGroupItem>
 * </ToggleGroup>
 * ```
 */
export * from './Toggle';
export * from './ToggleGroup';
export * from './ToggleGroupItem';
export * from './types';
export { useToggleGroup } from './ToggleGroupContext';
