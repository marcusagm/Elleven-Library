/**
 * Dropdown Menu component
 *
 * @module DropdownMenu
 * @description
 * The DropdownMenu component is a full-featured dropdown menu with support for submenus, checkboxes, radios,
 * and keyboard navigation.
 *
 * @example
 * ```tsx
 * import { DropdownMenu } from '@/components/ui';
 *
 * <DropdownMenu
 *   trigger={<Button>Open Menu</Button>}
 *   items={[
 *     { label: 'Item 1', onClick: () => console.log('Item 1') },
 *     { label: 'Item 2', onClick: () => console.log('Item 2') },
 *   ]}
 *   align="start"
 *   side="bottom"
 *   radioValue="item1"
 *   onRadioChange={(value) => console.log(value)}
 *   class="custom-class"
 *   contentClass="custom-content-class"
 * />
 * ```
 */

export * from './types';
export { DropdownMenu } from './DropdownMenu';
export { useMenuNavigation } from './useMenuNavigation';
export { useMenuPositioning } from './useMenuPositioning';
