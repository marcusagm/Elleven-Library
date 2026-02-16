/**
 * useCommand Hook
 * specific hook to subscribe to a command without registering a shortcut
 */

import { onCleanup } from 'solid-js';
import { onCommand } from '../dispatcher';
import { ShortcutPayload } from '../types';

export function useCommand(command: string, handler: (payload: ShortcutPayload) => void) {
  const unsub = onCommand(command, handler);
  onCleanup(unsub);
}

export function useCommands(commands: Record<string, (payload: ShortcutPayload) => void>) {
    const unsubs: (() => void)[] = [];

    for (const [command, handler] of Object.entries(commands)) {
        unsubs.push(onCommand(command, handler));
    }

    onCleanup(() => {
        unsubs.forEach(u => u());
    });
}
