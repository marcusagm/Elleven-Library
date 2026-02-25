/**
 * Settings Modal
 * A modal with sidebar navigation for application settings
 */

import { Component, createSignal, For, Show } from 'solid-js';
import { Keyboard, Palette, Settings, Info } from 'lucide-solid';
import { cn } from '../../../lib/utils';
import { Modal } from '../../ui';
import { KeyboardShortcutsPanel } from './KeyboardShortcutsPanel';
import { GeneralPanel } from './GeneralPanel';
import { AppearancePanel } from './AppearancePanel';
import { FoldersPanel } from './FoldersPanel';
import { AboutPanel } from './AboutPanel';
import './settings-modal.css';

/**
 * Valid settings tabs.
 */
export type SettingsTab = 'general' | 'appearance' | 'keyboard-shortcuts' | 'folders' | 'about';

/**
 * Definition structure for a settings tab.
 */
interface SettingsTabDefinition {
    /** Unique identifier for the tab. */
    identifier: SettingsTab;
    /** Human-readable label for the tab. */
    label: string;
    /** Icon component for the tab. */
    icon: Component<{ size?: number }>;
}

/**
 * Collection of all available settings tabs.
 */
const SETTINGS_TABS: SettingsTabDefinition[] = [
    { identifier: 'general', label: 'General', icon: Settings },
    { identifier: 'appearance', label: 'Appearance', icon: Palette },
    { identifier: 'keyboard-shortcuts', label: 'Keyboard Shortcuts', icon: Keyboard },
    { identifier: 'about', label: 'About', icon: Info }
];

/**
 * Properties for the SettingsModal component.
 */
export interface SettingsModalProperties {
    /** Whether the settings modal is currently open. */
    isOpen: boolean;
    /** Callback invoked when the modal requests closure. */
    onClose: () => void;
    /** The tab to show when the modal first opens. */
    initialTab?: SettingsTab;
}

/**
 * A modal providing access to various application-wide configurations,
 * organized into a sidebar-navigated layout.
 *
 * @param componentProperties - Properties for the component.
 * @returns The rendered SettingsModal.
 */
export const SettingsModal: Component<SettingsModalProperties> = componentProperties => {
    const [activeTab, setActiveTab] = createSignal<SettingsTab>(
        componentProperties.initialTab || 'general'
    );

    return (
        <Modal
            isOpen={componentProperties.isOpen}
            onClose={componentProperties.onClose}
            title="Settings"
            size="xl"
            class="settings-modal-wrapper"
        >
            <div class="settings-modal-content">
                {/* Sidebar */}
                <nav class="settings-sidebar" aria-label="Settings navigation">
                    <ul class="settings-sidebar-list">
                        <For each={SETTINGS_TABS}>
                            {tab => (
                                <li>
                                    <button
                                        type="button"
                                        class={cn(
                                            'settings-sidebar-item',
                                            activeTab() === tab.identifier && 'is-active'
                                        )}
                                        onClick={() => setActiveTab(tab.identifier)}
                                        aria-current={
                                            activeTab() === tab.identifier ? 'page' : undefined
                                        }
                                    >
                                        <tab.icon size={16} />
                                        <span>{tab.label}</span>
                                    </button>
                                </li>
                            )}
                        </For>
                    </ul>
                </nav>

                {/* Panel Content */}
                <div class="settings-panel">
                    <Show when={activeTab() === 'general'}>
                        <GeneralPanel />
                    </Show>
                    <Show when={activeTab() === 'appearance'}>
                        <AppearancePanel />
                    </Show>
                    <Show when={activeTab() === 'keyboard-shortcuts'}>
                        <KeyboardShortcutsPanel />
                    </Show>
                    <Show when={activeTab() === 'folders'}>
                        <FoldersPanel />
                    </Show>
                    <Show when={activeTab() === 'about'}>
                        <AboutPanel />
                    </Show>
                </div>
            </div>
        </Modal>
    );
};
