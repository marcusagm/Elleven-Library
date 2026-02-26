import { Component, Show, splitProps } from 'solid-js';
import { ProgressBar } from '../ProgressBar';
import { cn as concatenateClasses } from '../../../lib/utils';
import { LoaderProperties } from './types';
import './loader.css';

/**
 * Loader component for displaying various loading states and indicators.
 * Can display a card animation, text, and an optional progress bar.
 *
 * @param {LoaderProperties} properties - Component properties.
 * @returns {JSX.Element} The rendered loader component.
 *
 * @example
 * <Loader isFullscreen text="Saving changes..." />
 *
 * @example
 * <Loader size="sm" progress={45} maximumValue={100} />
 */
export const Loader: Component<LoaderProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'size',
        'isFullscreen',
        'text',
        'progress',
        'maximumValue',
        'class'
    ]);

    const activeSize = () => localProperties.size || 'md';

    return (
        <div
            class={concatenateClasses(
                'loader-container',
                localProperties.isFullscreen && 'fullscreen',
                localProperties.class
            )}
            role="status"
            {...remainingProperties}
        >
            <div class={concatenateClasses('loader-cards', activeSize())}>
                <div class="loader-card card-3" />
                <div class="loader-card card-2" />
                <div class="loader-card card-1" />
            </div>

            <Show when={localProperties.text}>
                <span class="loader-text">{localProperties.text}</span>
            </Show>

            <Show when={typeof localProperties.progress === 'number'}>
                <div class="loader-progress-wrapper">
                    <ProgressBar
                        value={localProperties.progress!}
                        max={localProperties.maximumValue}
                        size="sm"
                    />
                </div>
            </Show>
        </div>
    );
};
