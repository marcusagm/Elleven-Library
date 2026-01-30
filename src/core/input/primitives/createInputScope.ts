/**
 * createInputScope Primitive
 * Manages scope lifecycle for components
 */

import { onCleanup, createEffect } from 'solid-js';
import { inputStore } from '../store/inputStore';
import type { InputScopeName } from '../types';
import { SCOPE_PRIORITIES } from '../types';

/**
 * Push a scope when component mounts, pop when it unmounts
 */
export function createInputScope(
  name: InputScopeName, 
  priority?: number, 
  blockLowerScopes?: boolean
): void {
  const resolvedPriority = priority ?? SCOPE_PRIORITIES[name] ?? 0;
  
  // Push scope immediately during component creation
  inputStore.pushScope(name, resolvedPriority, blockLowerScopes);
  
  // Cleanup when component unmounts
  onCleanup(() => {
    inputStore.popScope(name);
  });
}

/**
 * Temporarily push a scope based on a condition (Reactive)
 */
export function createConditionalScope(
  name: InputScopeName, 
  when: () => boolean,
  priority?: number,
  blockLowerScopes?: boolean
): void {
  const resolvedPriority = priority ?? SCOPE_PRIORITIES[name] ?? 0;
  
  createEffect(() => {
    if (when()) {
      inputStore.pushScope(name, resolvedPriority, blockLowerScopes);
      onCleanup(() => {
        inputStore.popScope(name);
      });
    }
  });
}

/**
 * HOC/Wrapper component for scope management
 */
export function useScopeOnMount(
  name: InputScopeName, 
  priority?: number, 
  blockLowerScopes?: boolean
): void {
  createInputScope(name, priority, blockLowerScopes);
}
