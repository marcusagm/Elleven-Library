/**
 * TreeView Component
 *
 * @module TreeView
 * @description
 * A hierarchical navigation structure with support for selection, expansion,
 * drag-and-drop, inline editing, and keyboard navigation.
 *
 * @example
 * ```tsx
 * import { TreeView } from '@/components/ui';
 *
 * <TreeView
 *   items={[
 *     {
 *       id: '1',
 *       label: 'Node 1',
 *       children: [
 *         {
 *           id: '1-1',
 *           label: 'Node 1-1'
 *         },
 *         {
 *           id: '1-2',
 *           label: 'Node 1-2'
 *         }
 *       ]
 *     },
 *     {
 *       id: '2',
 *       label: 'Node 2'
 *     }
 *   ]}
 *   class="custom-class"
 *   indentSize={20}
 *   draggable={true}
 *   dragType="tree-node"
 *   acceptedDragTypes={['tree-node']}
 * />
 * ```
 */

export { TreeView } from './TreeView';
export type { TreeNode, TreeViewProps } from './types';
