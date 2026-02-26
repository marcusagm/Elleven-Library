import { Component, splitProps, Show, createMemo } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { createId as generateUniqueId } from '../../../lib/primitives/createId';
import { ProgressBarProperties } from './types';
import './progress-bar.css';

/**
 * ProgressBar component for indicating the completion status of a task or an ongoing loading process.
 * Supports discrete progress values as well as indeterminate loading animations.
 *
 * @param {ProgressBarProperties} properties - Properties for the progress bar.
 * @returns {JSX.Element} A stylized progress bar element.
 *
 * @example
 * <ProgressBar value={60} isLabelVisible labelTitle="Uploading files..." />
 *
 * @example
 * <ProgressBar isIndeterminate size="lg" />
 */
export const ProgressBar: Component<ProgressBarProperties> = properties => {
    const [localProperties] = splitProps(properties, [
        'value',
        'maximumValue',
        'size',
        'isLabelVisible',
        'labelTitle',
        'isIndeterminate',
        'class'
    ]);

    /**
     * Unique identifier for linking the label aria-labelledby for accessibility.
     */
    const accessibilityIdentifier = generateUniqueId('progress');

    /**
     * Identifier for the label text to be referenced by the progress bar container.
     */
    const labelIdentifier = `${accessibilityIdentifier}-label`;

    /**
     * Resolves the maximum possible value for the progress, defaulting to 100.
     */
    const resolvedMaximumValue = () => localProperties.maximumValue ?? 100;

    /**
     * Resolves the size variant to use, defaulting to 'sm'.
     */
    const activeSize = () => localProperties.size ?? 'sm';

    /**
     * Calculates the percentage representation of the current progress.
     */
    const progressPercentage = createMemo(() => {
        const clampedValue = Math.max(0, Math.min(localProperties.value, resolvedMaximumValue()));
        return (clampedValue / resolvedMaximumValue()) * 100;
    });

    return (
        <div
            class={concatenateClasses('ui-progress-container', localProperties.class)}
            role="progressbar"
            aria-valuenow={localProperties.isIndeterminate ? undefined : localProperties.value}
            aria-valuemin={0}
            aria-valuemax={resolvedMaximumValue()}
            aria-labelledby={localProperties.labelTitle ? labelIdentifier : undefined}
            aria-busy={localProperties.isIndeterminate}
        >
            <Show when={localProperties.isLabelVisible}>
                <div class="ui-progress-label">
                    <Show when={localProperties.labelTitle}>
                        <span id={labelIdentifier}>{localProperties.labelTitle}</span>
                    </Show>
                    <Show when={!localProperties.isIndeterminate}>
                        <span class="ui-progress-percentage">
                            {Math.round(progressPercentage())}%
                        </span>
                    </Show>
                </div>
            </Show>

            <div class={concatenateClasses('ui-progress-track', `ui-progress-${activeSize()}`)}>
                <div
                    class={concatenateClasses(
                        'ui-progress-fill',
                        localProperties.isIndeterminate && 'ui-progress-indeterminate'
                    )}
                    style={
                        localProperties.isIndeterminate
                            ? undefined
                            : { width: `${progressPercentage()}%` }
                    }
                />
            </div>
        </div>
    );
};
