/**
 * Resizable components
 *
 * @module Resizable
 * @description
 * The Resizable component is a specialized component for creating resizable layouts.
 *
 * @example
 * ```tsx
 * import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '@/components/ui';
 *
 * <ResizablePanelGroup orientation="horizontal">
 *  <ResizablePanel>
 *    <p>Panel 1</p>
 *  </ResizablePanel>
 *  <ResizableHandle />
 *  <ResizablePanel>
 *    <p>Panel 2</p>
 *  </ResizablePanel>
 * </ResizablePanelGroup>
 * ```
 */
export * from './ResizableRoot';
export * from './ResizablePanel';
export * from './ResizableHandle';
export * from './types';
