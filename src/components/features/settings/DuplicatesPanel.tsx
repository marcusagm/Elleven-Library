import { Component, createSignal } from 'solid-js';
import { Button, SectionGroup, Switch } from '../../ui';

/**
 * Settings panel for configuring Duplicate File Detection.
 */
export const DuplicatesPanel: Component = () => {
    const [autoDetect, setAutoDetect] = createSignal(true);
    const [perceptualThreshold, setPerceptualThreshold] = createSignal(95);

    // TODO: In a real implementation, we would load and save these settings
    // from/to the backend configuration store or a local store.

    return (
        <div class="settings-panel-content duplicates-panel">
            <h2 class="settings-panel-title">Duplicate Detection</h2>
            <p style={{ 'margin-bottom': '24px', color: 'var(--text-secondary)' }}>
                Configure how Mundam scans and identifies duplicate assets.
            </p>

            <SectionGroup
                title="Background Scanning"
                description="Settings related to real-time exact duplicate detection."
            >
                <div
                    class="general-setting-row"
                    style={{
                        display: 'flex',
                        'justify-content': 'space-between',
                        'align-items': 'center'
                    }}
                >
                    <div style={{ display: 'flex', 'flex-direction': 'column', gap: '4px' }}>
                        <span class="setting-label">Auto-Detect Exact Duplicates on Import</span>
                        <span style={{ color: 'var(--text-tertiary)', 'font-size': '0.85rem' }}>
                            Instantly check and group exact copies as soon as they are added to the
                            library. Highly recommended.
                        </span>
                    </div>
                    <Switch checked={autoDetect()} onchange={setAutoDetect} />
                </div>
            </SectionGroup>

            <SectionGroup
                title="Deep Scan Rules"
                description="Configuration for perceptual hashing and similarity checks."
            >
                <div
                    class="general-setting-row"
                    style={{
                        display: 'flex',
                        'justify-content': 'space-between',
                        'align-items': 'center'
                    }}
                >
                    <div style={{ display: 'flex', 'flex-direction': 'column', gap: '4px' }}>
                        <span class="setting-label">Perceptual Similarity Threshold</span>
                        <span style={{ color: 'var(--text-tertiary)', 'font-size': '0.85rem' }}>
                            The minimum confidence score required to consider two different images
                            as duplicates (e.g. resized variants).
                        </span>
                    </div>
                    <div style={{ display: 'flex', 'align-items': 'center', gap: '8px' }}>
                        <input
                            type="range"
                            min="80"
                            max="100"
                            value={perceptualThreshold()}
                            onInput={e =>
                                setPerceptualThreshold(parseInt(e.currentTarget.value, 10))
                            }
                        />
                        <span>{perceptualThreshold()}%</span>
                    </div>
                </div>
            </SectionGroup>

            <SectionGroup
                title="Maintenance"
                description="Erase all existing hashes and re-analyze the entire library. Use this only if you suspect corruption or if rules changed significantly."
            >
                <div class="general-setting-row">
                    <Button variant="destructive" size="sm">
                        Rebuild Index
                    </Button>
                </div>
            </SectionGroup>
        </div>
    );
};
