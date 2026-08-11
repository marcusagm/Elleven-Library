/**
 * TitleBar
 *
 * @module TitleBar
 * @description
 * Custom window title bar component that replaces the native OS title bar.
 * Provides platform-aware window controls (macOS traffic lights or custom buttons
 * for Windows/Linux), application view navigation, and a draggable region for
 * repositioning the window.
 *
 * @example
 * ```tsx
 * import { TitleBar } from '@/components/layout/TitleBar';
 *
 * <TitleBar
 *   activeView="gallery"
 *   onViewChange={(view) => setActiveView(view)}
 * />
 * ```
 */
export * from './types';
export { TitleBar } from './TitleBar';
export { WindowControls } from './WindowControls';
