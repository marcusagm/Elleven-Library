/**
 * Shortcut Store
 * Reactive state for registered shortcuts
 */

import { createSignal, createEffect } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import type {
    ShortcutDefinition,
    RegisteredShortcut,
    ShortcutActions,
    InputScopeName,
    SerializedShortcut,
    InputToken
} from '../types';
import { normalizeKeysToTokens, canonicalizeShortcut } from '../normalizer';
import { DEFAULT_SHORTCUTS } from './defaults';

function createShortcutStore() {
    const [shortcuts, setShortcuts] = createSignal<Map<string, RegisteredShortcut>>(new Map());
    const [nextIdValue, setNextIdValue] = createSignal(1);
    const [customizations, setCustomizations] = createSignal<Map<string, string>>(new Map());

    function generateShortcutId(): string {
        const id = `sc_${nextIdValue()}`;
        setNextIdValue(previous => previous + 1);
        return id;
    }

    function resolveFinalKeys(definition: ShortcutDefinition, currentId: string): string {
        const customKeys = customizations().get(currentId);
        if (customKeys) return customKeys;

        if (typeof definition.keys === 'string') return definition.keys;
        return definition.keys.join(' ');
    }

    function calculateIgnoreInputs(definition: ShortcutDefinition, tokens: InputToken[]): boolean {
        if (definition.ignoreInputs !== undefined) return definition.ignoreInputs;

        const hasModifiers = tokens.some(token => {
            const modifiers = token.meta?.modifiers;
            return Array.isArray(modifiers) && modifiers.length > 0;
        });

        return !hasModifiers;
    }

    function createRegisteredShortcut(
        definition: ShortcutDefinition,
        handler?: ShortcutDefinition['handler']
    ): RegisteredShortcut {
        const currentId = definition.id || generateShortcutId();
        const finalHandler = handler || definition.handler;
        const finalKeys = resolveFinalKeys(definition, currentId);
        const tokens = normalizeKeysToTokens(finalKeys);

        return {
            ...definition,
            id: currentId,
            handler: finalHandler,
            keys: finalKeys,
            tokens,
            normalizedKeys: canonicalizeShortcut(finalKeys),
            scope: definition.scope || 'global',
            priority: definition.priority ?? 0,
            ignoreInputs: calculateIgnoreInputs(definition, tokens),
            preventDefault: definition.preventDefault ?? true,
            isDefault: definition.isDefault ?? true
        };
    }

    async function saveShortcutsToBackend() {
        try {
            const data: Record<string, string> = {};
            const currentCustomizations = customizations();
            const allShortcuts = shortcuts();

            for (const [id, keys] of currentCustomizations) {
                const shortcut = allShortcuts.get(id);
                if (shortcut) {
                    const storageKey = `${shortcut.name}::${shortcut.scope || 'global'}`;
                    data[storageKey] = keys;
                }
            }

            await invoke('set_setting', { key: 'shortcuts', value: data });
        } catch (error) {
            console.warn('[ShortcutStore] Failed to save shortcuts:', error);
        }
    }

    const actions: ShortcutActions = {
        register: (definition: ShortcutDefinition, handler?: ShortcutDefinition['handler']) => {
            const registered = createRegisteredShortcut(definition, handler);

            setShortcuts(previous => {
                const next = new Map(previous);
                next.set(registered.id, registered);
                return next;
            });

            return registered.id;
        },

        unregister: (id: string) => {
            setShortcuts(previous => {
                const next = new Map(previous);
                next.delete(id);
                return next;
            });
        },

        edit: (id: string, newKeys: string, persist = true) => {
            const current = shortcuts().get(id);
            if (!current) {
                console.warn(`[ShortcutStore] Cannot edit: shortcut ${id} not found`);
                return;
            }

            setCustomizations(previous => {
                const next = new Map(previous);
                next.set(id, newKeys);
                return next;
            });

            const tokens = normalizeKeysToTokens(newKeys);
            const updated: RegisteredShortcut = {
                ...current,
                keys: newKeys,
                tokens,
                normalizedKeys: canonicalizeShortcut(newKeys),
                isDefault: false
            };

            setShortcuts(previous => {
                const next = new Map(previous);
                next.set(id, updated);
                return next;
            });

            if (persist) saveShortcutsToBackend();
        },

        resetToDefault: (id: string) => {
            const defaultDefinition = DEFAULT_SHORTCUTS.find(
                definition => definition.id === id || definition.name === shortcuts().get(id)?.name
            );

            if (!defaultDefinition) {
                console.warn(`[ShortcutStore] Cannot reset: no default found for ${id}`);
                return;
            }

            setCustomizations(previous => {
                const next = new Map(previous);
                next.delete(id);
                return next;
            });

            const registered = createRegisteredShortcut({ ...defaultDefinition, id });
            setShortcuts(previous => {
                const next = new Map(previous);
                next.set(id, registered);
                return next;
            });

            saveShortcutsToBackend();
        },

        resetAllToDefaults: () => {
            setCustomizations(new Map());

            const newShortcuts = new Map<string, RegisteredShortcut>();
            for (const definition of DEFAULT_SHORTCUTS) {
                const registered = createRegisteredShortcut(definition);
                newShortcuts.set(registered.id, registered);
            }
            setShortcuts(newShortcuts);
            saveShortcutsToBackend();
        },

        list: () => {
            return Array.from(shortcuts().values());
        },

        getByScope: (scope: InputScopeName) => {
            return Array.from(shortcuts().values()).filter(shortcut => shortcut.scope === scope);
        },

        detectConflicts: (keys: string, excludeId?: string, scope?: string) => {
            const normalized = canonicalizeShortcut(keys);
            const targetScope = scope || 'global';

            return Array.from(shortcuts().values())
                .filter(shortcut => {
                    if (shortcut.id === excludeId) return false;
                    if (shortcut.normalizedKeys !== normalized) return false;
                    const shortcutScope = shortcut.scope || 'global';
                    return shortcutScope === targetScope;
                })
                .map(shortcut => shortcut.name);
        }
    };

    // Initialization
    for (const definition of DEFAULT_SHORTCUTS) {
        actions.register(definition);
    }

    async function loadShortcutsFromBackend() {
        try {
            const saved = await invoke<Record<string, string> | null>('get_setting', {
                key: 'shortcuts'
            });
            if (saved) {
                for (const [key, keys] of Object.entries(saved)) {
                    const [name, scope] = key.split('::');
                    const found = Array.from(shortcuts().values()).find(
                        shortcut =>
                            shortcut.name === name &&
                            (shortcut.scope || 'global') === (scope || 'global')
                    );
                    if (found) {
                        actions.edit(found.id, keys, false);
                    }
                }
            }
        } catch (error) {
            console.warn('[ShortcutStore] Failed to load shortcuts:', error);
        }
    }

    createEffect(() => {
        loadShortcutsFromBackend();
    });

    return {
        shortcuts,
        customizations,
        ...actions,
        getById: (id: string) => shortcuts().get(id),
        getByCommand: (command: string) =>
            Array.from(shortcuts().values()).find(s => s.command === command),
        getByNameAndScope: (name: string, scope: InputScopeName = 'global') =>
            Array.from(shortcuts().values()).find(
                s => s.name === name && (s.scope || 'global') === (scope || 'global')
            ),
        getCategories: () => {
            const categories = new Set<string>();
            for (const shortcut of shortcuts().values()) {
                if (shortcut.category) categories.add(shortcut.category);
            }
            return Array.from(categories).sort();
        },
        serialize: (): SerializedShortcut[] =>
            Array.from(shortcuts().values()).map(shortcut => ({
                id: shortcut.id,
                name: shortcut.name,
                description: shortcut.description,
                keys: shortcut.keys as string,
                scope: shortcut.scope || 'global',
                category: shortcut.category,
                isCustom: !shortcut.isDefault
            })),
        getDefault: (id: string): ShortcutDefinition | undefined => {
            const current = shortcuts().get(id);
            if (!current) return undefined;
            return DEFAULT_SHORTCUTS.find(
                definition => definition.id === id || definition.name === current.name
            );
        }
    };
}

export const shortcutStore = createShortcutStore();
