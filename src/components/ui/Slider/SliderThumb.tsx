import { Component, JSX, createSignal, splitProps, createMemo, Show, untrack } from 'solid-js';
import { cn } from '../../../lib/utils';
import { useSlider } from './SliderContext';
import { SliderTooltip } from './SliderTooltip.tsx';
import { createConditionalScope } from '../../../core/input/primitives/createInputScope';
import { useShortcut } from '../../../core/input/primitives/createShortcut';

/**
 * Properties for the SliderThumb component.
 */
interface SliderThumbProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** Controls whether an interactive tooltip is displayed above the thumb with the current value. */
    showTooltip?: boolean;
}

/**
 * The SliderThumb component represents the interactive handle that users drag to select a value.
 * It manages its own focus state, accessibility attributes (ARIA), and integrates with the
 * global input system to provide standard keyboard navigation (Arrow keys, Home, End, PageUp/Down).
 *
 * This component acts as the primary focusable element for the slider system.
 *
 * @param componentProperties - Properties for the SliderThumb.
 * @returns The rendered thumb handle element.
 */
export const SliderThumb: Component<SliderThumbProperties> = componentProperties => {
    const [localProperties, otherProperties] = splitProps(componentProperties, [
        'class',
        'showTooltip'
    ]);
    const slider = useSlider();

    /** Track whether the handle currently has keyboard focus. */
    const [isFocused, setIsFocused] = createSignal(false);

    /**
     * Unique scope identifier for this slider instance's keyboard shortcuts.
     * Ensures that multiple sliders on the same page don't conflict.
     */
    const sliderInputScope = createMemo(() => `slider-${slider.sliderIdentifier()}`);

    /**
     * Cache the initial scope name for use in shortcut registration.
     * Shortcuts are registered once; using a stable identifier avoids reactivity warnings.
     */
    const initialInputScopeIdentifier = untrack(() => sliderInputScope());

    // Enable the custom keyboard shortcut scope only when this specific thumb is focused.
    createConditionalScope(initialInputScopeIdentifier, isFocused, 1200);

    /**
     * Utility to ensure unknown new value remains strictly between minimum and maximum bounds.
     *
     * @param targetValue - The numeric value to be clamped.
     * @returns The clamped value.
     */
    const clampValueToRange = (targetValue: number) => {
        return Math.min(slider.maximumValue(), Math.max(slider.minimumValue(), targetValue));
    };

    /**
     * Calculates a 'large' increment, typically used for PageUp/PageDown interactions.
     * By convention, this is 10 times the base step value.
     */
    const calculateLargeStepIncrement = () => slider.stepValue() * 10;

    // --- Keyboard Shortcut Registration ---

    // Standard Increments (ArrowRight, ArrowUp)
    useShortcut(
        ['ArrowRight', 'ArrowUp'],
        event => {
            event?.preventDefault();
            const newValue = clampValueToRange(slider.value() + slider.stepValue());
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialInputScopeIdentifier, system: true }
    );

    // Standard Decrements (ArrowLeft, ArrowDown)
    useShortcut(
        ['ArrowLeft', 'ArrowDown'],
        event => {
            event?.preventDefault();
            const newValue = clampValueToRange(slider.value() - slider.stepValue());
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialInputScopeIdentifier, system: true }
    );

    // Large Increments (PageUp)
    useShortcut(
        'PageUp',
        event => {
            event?.preventDefault();
            const newValue = clampValueToRange(slider.value() + calculateLargeStepIncrement());
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialInputScopeIdentifier, system: true }
    );

    // Large Decrements (PageDown)
    useShortcut(
        'PageDown',
        event => {
            event?.preventDefault();
            const newValue = clampValueToRange(slider.value() - calculateLargeStepIncrement());
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialInputScopeIdentifier, system: true }
    );

    // Jump to Minimum (Home)
    useShortcut(
        'Home',
        event => {
            event?.preventDefault();
            const newValue = slider.minimumValue();
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialInputScopeIdentifier, system: true }
    );

    // Jump to Maximum (End)
    useShortcut(
        'End',
        event => {
            event?.preventDefault();
            const newValue = slider.maximumValue();
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialInputScopeIdentifier, system: true }
    );

    /** Updates local focus state to active. */
    const handleFocus = () => setIsFocused(true);
    /** Updates local focus state to inactive. */
    const handleBlur = () => setIsFocused(false);

    return (
        <div
            class={cn(
                'ui-slider-thumb',
                slider.isDragging() && 'ui-slider-thumb-active',
                localProperties.class
            )}
            style={{
                [slider.orientation() === 'vertical' ? 'bottom' : 'left']: `${slider.percentage()}%`
            }}
            role="slider"
            tabindex={slider.isDisabled() ? -1 : 0}
            aria-valuemin={slider.minimumValue()}
            aria-valuemax={slider.maximumValue()}
            aria-valuenow={slider.value()}
            aria-valuetext={slider.formatValue(slider.value())}
            aria-orientation={slider.orientation()}
            aria-disabled={slider.isDisabled()}
            onFocus={handleFocus}
            onBlur={handleBlur}
            {...otherProperties}
        >
            <Show when={localProperties.showTooltip}>
                <SliderTooltip />
            </Show>
        </div>
    );
};
