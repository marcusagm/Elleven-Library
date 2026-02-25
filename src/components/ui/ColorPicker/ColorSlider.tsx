import { Component, onCleanup } from 'solid-js';
import { useColorPickerContext } from './context';

/**
 * Slider component for selecting Hue.
 * Displays a colorful spectrum gradient with a draggable thumb.
 *
 * @returns {import('solid-js').JSX.Element} The rendered hue slider component.
 */
export const ColorSlider: Component = () => {
    const {
        hueSaturationBrightness,
        updateColorFromHueSaturationBrightness,
        setIsDragging,
        activeColor
    } = useColorPickerContext();

    let sliderReference: HTMLDivElement | undefined;

    /**
     * Calculates and updates the hue based on pointer coordinates within the slider.
     *
     * @param {MouseEvent | PointerEvent} pointerEvent - The movement event.
     */
    const handleMove = (pointerEvent: MouseEvent | PointerEvent) => {
        if (!sliderReference) return;

        const boundingRectangle = sliderReference.getBoundingClientRect();
        const horizontalPosition = Math.max(
            0,
            Math.min(boundingRectangle.width, pointerEvent.clientX - boundingRectangle.left)
        );
        const hue = (horizontalPosition / boundingRectangle.width) * 360;

        updateColorFromHueSaturationBrightness({ hue });
    };

    const handleGlobalMove = (event: MouseEvent) => handleMove(event);

    const handleGlobalUp = () => {
        setIsDragging(false);
        document.removeEventListener('mousemove', handleGlobalMove);
        document.removeEventListener('mouseup', handleGlobalUp);
    };

    const handleMouseDown = (event: MouseEvent) => {
        setIsDragging(true);
        handleMove(event);
        document.addEventListener('mousemove', handleGlobalMove);
        document.addEventListener('mouseup', handleGlobalUp);
    };

    /**
     * Handles keyboard navigation for fine-tuning the hue value.
     *
     * @param {KeyboardEvent} keyboardEvent - The keyboard event.
     */
    const handleKeyDown = (keyboardEvent: KeyboardEvent) => {
        const movementStep = keyboardEvent.shiftKey ? 10 : 1;
        const current = hueSaturationBrightness();

        switch (keyboardEvent.key) {
            case 'ArrowRight':
                keyboardEvent.preventDefault();
                updateColorFromHueSaturationBrightness({ hue: (current.hue + movementStep) % 360 });
                break;
            case 'ArrowLeft':
                keyboardEvent.preventDefault();
                updateColorFromHueSaturationBrightness({
                    hue: (current.hue - movementStep + 360) % 360
                });
                break;
        }
    };

    onCleanup(() => {
        document.removeEventListener('mousemove', handleGlobalMove);
        document.removeEventListener('mouseup', handleGlobalUp);
    });

    return (
        <div
            ref={sliderReference}
            class="ui-color-picker-hue"
            onMouseDown={handleMouseDown}
            onKeyDown={handleKeyDown}
            tabindex={0}
            role="slider"
            aria-label="Hue"
            aria-valuemin={0}
            aria-valuemax={360}
            aria-valuenow={Math.round(hueSaturationBrightness().hue)}
        >
            <div
                class="ui-color-picker-hue-thumb"
                style={{
                    left: `${(hueSaturationBrightness().hue / 360) * 100}%`,
                    display: activeColor() === 'transparent' ? 'none' : 'block'
                }}
            />
        </div>
    );
};
