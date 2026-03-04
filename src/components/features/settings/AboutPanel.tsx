import { Component, For, createMemo } from 'solid-js';
import { openUrl } from '@tauri-apps/plugin-opener';
import { appearance } from '../../../core/store/appearanceStore';
import logoColor from '../../../assets/logo-color.svg';
import logoWhite from '../../../assets/logo-white.svg';
import './about-panel.css';

import { TECH_STACK } from './data/tech-stack';
export const AboutPanel: Component = () => {
    const handleOpenLink = (url: string) => {
        openUrl(url).catch((err: unknown) => console.error('Failed to open link:', err));
    };

    const effectiveLogo = createMemo(() => {
        let mode = appearance().mode;
        if (mode === 'system') {
            mode = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
        }
        return mode === 'dark' ? logoWhite : logoColor;
    });

    return (
        <div class="settings-panel-content about-panel">
            {/* Hero Section */}
            <div class="about-hero">
                <div class="about-logo-container">
                    <img
                        id="about-app-logo"
                        src={effectiveLogo()}
                        alt="Mundam Logo"
                        class="about-logo"
                    />
                </div>
                {/* <h1 class="about-app-name">Mundam</h1> */}
                <div class="about-version">Version 0.1.0</div>
            </div>

            {/* Tech Stack */}
            <div class="about-sections">
                <For each={TECH_STACK}>
                    {category => (
                        <div class="about-category">
                            <h3 class="about-section-title">{category.title}</h3>
                            <div class="tech-grid">
                                <For each={category.items}>
                                    {tech => (
                                        <button
                                            class="tech-card"
                                            onClick={() => handleOpenLink(tech.url)}
                                            title={`Open ${tech.name} website`}
                                        >
                                            <span class="tech-name">{tech.name}</span>
                                            <span class="tech-desc">{tech.description}</span>
                                        </button>
                                    )}
                                </For>
                            </div>
                        </div>
                    )}
                </For>
            </div>

            {/* Footer */}
            <footer class="about-footer">
                <p class="about-copyright">© 2024-2025 Marcus Maia. All rights reserved.</p>
                <div class="about-links">
                    <button
                        onClick={() => handleOpenLink('https://github.com/marcusagm/Mundam')}
                        class="about-link"
                    >
                        GitHub
                    </button>
                    <button onClick={() => handleOpenLink('https://mundam.app')} class="about-link">
                        Website
                    </button>
                    <button
                        onClick={() => handleOpenLink('https://github.com/marcusagm/Mundam/issues')}
                        class="about-link"
                    >
                        Support
                    </button>
                </div>
            </footer>
        </div>
    );
};
