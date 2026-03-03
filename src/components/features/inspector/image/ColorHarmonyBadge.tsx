import { Component, Show } from 'solid-js';
import { Palette } from 'lucide-solid';
import { type HarmonyType, HARMONY_DISPLAY_MAP } from './colorHarmonyUtils';
import { Tooltip } from '../../../ui';

interface ColorHarmonyBadgeProperties {
    harmonyType: HarmonyType;
}

/**
 * Visual badge showing the detected color harmony classification.
 * Displays a tooltip with a brief explanation of the harmony type on hover.
 */
export const ColorHarmonyBadge: Component<ColorHarmonyBadgeProperties> = properties => {
    const displayInfo = () => HARMONY_DISPLAY_MAP[properties.harmonyType];

    return (
        <Show when={displayInfo()}>
            <Tooltip content={displayInfo().description}>
                <div class="color-harmony-badge">
                    <span class="harmony-icon">
                        <Palette size={12} />
                    </span>
                    <span class="harmony-label">{displayInfo().label}</span>
                </div>
            </Tooltip>
        </Show>
    );
};
