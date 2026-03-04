/**
 * ColorPicker component
 *
 * @module ColorPicker
 * @description
 * The ColorPicker component is a modular color selection system using the Compound Component pattern.
 * It provides a graphical picker for color selection with support for various color formats.
 *
 * @example
 * ```tsx
 * import { ColorPicker } from '@/components/ui';
 *
 * <ColorPicker
 *   class="custom-class"
 *   allowNoColor={false}
 *   showInput={true}
 *   color="#ff0000"
 *   onChange={(value) => console.log(value)}
 *   presets={["#ff0000", "#00ff00", "#0000ff"]}
 * />
 * ```
 */
export * from './Root';
export * from './types';
export { useColorPicker } from './useColorPicker';
