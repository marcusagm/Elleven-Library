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
    /** Whether to show a tooltip above the thumb. */
    showTooltip?: boolean;
}

/**
 * The interactive handle for the slider.
 * It manages focus, keyboard navigation (via the input system), and accessibility attributes.
 *
 * @param componentProperties - Properties for the SliderThumb.
 * @returns The rendered thumb element.
 */
export const SliderThumb: Component<SliderThumbProperties> = componentProperties => {
    const [localProperties, otherProperties] = splitProps(componentProperties, [
        'class',
        'showTooltip'
    ]);
    const slider = useSlider();
    const [isFocused, setIsFocused] = createSignal(false);

    // Register a conditional scope and shortcuts when this thumb is focused
    const sliderScope = createMemo(() => `slider-${slider.sliderIdentifier()}`);

    // Since shortcuts are registered once on mount, we use a stable scope name
    // derived from the initial identifier to avoid reactivity lint warnings
    // on initialization functions that aren't reactive.
    const initialScope = untrack(() => sliderScope());

    createConditionalScope(initialScope, isFocused, 1200);

    const clamp = (value: number) => {
        return Math.min(slider.maximumValue(), Math.max(slider.minimumValue(), value));
    };

    const bigStep = () => slider.stepValue() * 10;

    // Register accessibility shortcuts
    useShortcut(
        ['ArrowRight', 'ArrowUp'],
        event => {
            event?.preventDefault();
            const newValue = clamp(slider.value() + slider.stepValue());
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialScope }
    );

    useShortcut(
        ['ArrowLeft', 'ArrowDown'],
        event => {
            event?.preventDefault();
            const newValue = clamp(slider.value() - slider.stepValue());
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialScope }
    );

    useShortcut(
        'PageUp',
        event => {
            event?.preventDefault();
            const newValue = clamp(slider.value() + bigStep());
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialScope }
    );

    useShortcut(
        'PageDown',
        event => {
            event?.preventDefault();
            const newValue = clamp(slider.value() - bigStep());
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialScope }
    );

    useShortcut(
        'Home',
        event => {
            event?.preventDefault();
            const newValue = slider.minimumValue();
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialScope }
    );

    useShortcut(
        'End',
        event => {
            event?.preventDefault();
            const newValue = slider.maximumValue();
            slider.setValue(newValue);
            slider.commitValue(newValue);
        },
        { scope: initialScope }
    );

    const handleFocus = () => setIsFocused(true);
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
