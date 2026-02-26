/**
 * @module SidebarPanel
 * Specialized containers for navigation or informational side panels.
 *
 * @example
 * ```tsx
 * import { SidebarPanel } from '@/components/ui';
 *
 * <SidebarPanel
 *   title="Layers"
 *   headerActions={<Button icon="plus" />}
 *   footerContent={<Button icon="plus" />}
 *   class="my-class"
 *   contentClass="my-content-class"
 * >
 *   <LayerList />
 * </SidebarPanel>
 * ```
 */

export * from './SidebarPanel';
export * from './types';
