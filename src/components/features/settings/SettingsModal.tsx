/**
 * Settings Modal
 * A modal with sidebar navigation for application settings
 */

import { Component, createSignal, For, Show } from 'solid-js';
import { Portal } from 'solid-js/web';
import { 
  X, 
  Keyboard, 
  Palette, 
  Settings, 
  FolderOpen,
  Info 
} from 'lucide-solid';
import { cn } from '../../../lib/utils';
import { createFocusTrap } from '../../../lib/primitives';
import { createConditionalScope, useShortcuts } from '../../../core/input';
import { KeyboardShortcutsPanel } from './KeyboardShortcutsPanel';
import './settings-modal.css';

export type SettingsTab = 
  | 'general' 
  | 'appearance' 
  | 'keyboard-shortcuts' 
  | 'folders' 
  | 'about';

interface SettingsTabDef {
  id: SettingsTab;
  label: string;
  icon: Component<{ size?: number }>;
}

const SETTINGS_TABS: SettingsTabDef[] = [
  { id: 'general', label: 'General', icon: Settings },
  { id: 'appearance', label: 'Appearance', icon: Palette },
  { id: 'keyboard-shortcuts', label: 'Keyboard Shortcuts', icon: Keyboard },
  { id: 'folders', label: 'Folders', icon: FolderOpen },
  { id: 'about', label: 'About', icon: Info },
];

export interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  initialTab?: SettingsTab;
}

export const SettingsModal: Component<SettingsModalProps> = (props) => {
  const [activeTab, setActiveTab] = createSignal<SettingsTab>(
    props.initialTab || 'general'
  );
  
  // Container ref for focus trap
  let containerRef: HTMLDivElement | undefined;
  
  // Input System Integration
  createConditionalScope("modal", () => props.isOpen, undefined, true);
  createFocusTrap(() => containerRef, () => props.isOpen);
  
  useShortcuts([
    {
      name: "Close Settings",
      scope: "modal",
      keys: ["Escape"],
      enabled: () => props.isOpen,
      action: () => props.onClose(),
      ignoreInputs: false,
    }
  ]);

  const handleOverlayClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) {
      props.onClose();
    }
  };

  return (
    <Show when={props.isOpen}>
      <Portal>
        <div 
          class="settings-modal-overlay" 
          onClick={handleOverlayClick}
        />
        <div 
          ref={containerRef}
          class="settings-modal-container"
          role="dialog"
          aria-modal="true"
          aria-labelledby="settings-title"
        >
          {/* Header */}
          <header class="settings-modal-header">
            <h1 id="settings-title" class="settings-modal-title">Settings</h1>
            <button
              type="button"
              class="settings-modal-close"
              onClick={props.onClose}
              aria-label="Close settings"
            >
              <X size={18} />
            </button>
          </header>
          
          {/* Content */}
          <div class="settings-modal-content">
            {/* Sidebar */}
            <nav class="settings-sidebar" aria-label="Settings navigation">
              <ul class="settings-sidebar-list">
                <For each={SETTINGS_TABS}>
                  {(tab) => (
                    <li>
                      <button
                        type="button"
                        class={cn(
                          'settings-sidebar-item',
                          activeTab() === tab.id && 'is-active'
                        )}
                        onClick={() => setActiveTab(tab.id)}
                        aria-current={activeTab() === tab.id ? 'page' : undefined}
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
        </div>
      </Portal>
    </Show>
  );
};

// Placeholder panels - to be implemented
const GeneralPanel: Component = () => (
  <div class="settings-panel-content">
    <h2 class="settings-panel-title">General</h2>
    <p class="settings-panel-description">
      General application settings will be available here.
    </p>
  </div>
);

const AppearancePanel: Component = () => (
  <div class="settings-panel-content">
    <h2 class="settings-panel-title">Appearance</h2>
    <p class="settings-panel-description">
      Customize the look and feel of Elleven Library.
    </p>
  </div>
);

const FoldersPanel: Component = () => (
  <div class="settings-panel-content">
    <h2 class="settings-panel-title">Folders</h2>
    <p class="settings-panel-description">
      Manage your library folders and watched directories.
    </p>
  </div>
);

const AboutPanel: Component = () => (
  <div class="settings-panel-content">
    <h2 class="settings-panel-title">About Elleven Library</h2>
    <p class="settings-panel-description">
      Version 0.1.0 • Built with Tauri + SolidJS
    </p>
  </div>
);
