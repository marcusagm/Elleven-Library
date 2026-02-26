/**
 * @module Sonner
 * A modular toast notification system for Solid.js.
 * Built with performance and accessibility in mind, supporting stacking and hover expansion.
 *
 * @example
 * ```tsx
 * import { Toaster, toast } from '@/components/ui';
 *
 * // Root application setup
 * <Toaster position="bottom-right" />
 *
 * // Emitting notifications
 * toast.success("Connected to database");
 * toast.error("Connection failed", { description: "Timeout reached." });
 * ```
 */

export * from './types';
export * from './state';
export { Toaster } from './Toaster';

/**
 * Sonner is an alias for the Toaster component, matching the common library name.
 */
export { Toaster as Sonner } from './Toaster';
