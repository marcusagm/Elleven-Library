import { Component, onCleanup } from 'solid-js';
import { useColorPickerContext } from './context';

/**
 * Area component for selecting Saturation and Brightness.
 * Displays a colorful gradient box with a draggable thumb.
 *
 * @returns {import('solid-js').JSX.Element} The rendered color area component.
 */
export const ColorArea: Component = () => {
    const {
        hueSaturationBrightness,
        updateColorFromHueSaturationBrightness,
        setIsDragging,
        activeColor
    } = useColorPickerContext();

    let areaReference: HTMLDivElement | undefined;

    /**
     * Calculates and updates the color based on pointer coordinates within the area.
     *
     * @param {MouseEvent | PointerEvent} pointerEvent - The movement event.
     */
    const handleMove = (pointerEvent: MouseEvent | PointerEvent) => {
        if (!areaReference) return;

        const boundingRectangle = areaReference.getBoundingClientRect();
        const horizontalPosition = Math.max(
            0,
            Math.min(boundingRectangle.width, pointerEvent.clientX - boundingRectangle.left)
        );
        const verticalPosition = Math.max(
            0,
            Math.min(boundingRectangle.height, pointerEvent.clientY - boundingRectangle.top)
        );

        const saturation = (horizontalPosition / boundingRectangle.width) * 100;
        const brightness = 100 - (verticalPosition / boundingRectangle.height) * 100;

        updateColorFromHueSaturationBrightness({ saturation, brightness });
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
     * Handles keyboard navigation for fine-tuning saturation and brightness.
     *
     * @param {KeyboardEvent} keyboardEvent - The keyboard event.
     */
    const handleKeyDown = (keyboardEvent: KeyboardEvent) => {
        const movementStep = keyboardEvent.shiftKey ? 10 : 1;
        const current = hueSaturationBrightness();

        switch (keyboardEvent.key) {
            case 'ArrowRight':
                keyboardEvent.preventDefault();
                updateColorFromHueSaturationBrightness({
                    saturation: Math.min(100, current.saturation + movementStep)
                });
                break;
            case 'ArrowLeft':
                keyboardEvent.preventDefault();
                updateColorFromHueSaturationBrightness({
                    saturation: Math.max(0, current.saturation - movementStep)
                });
                break;
            case 'ArrowUp':
                keyboardEvent.preventDefault();
                updateColorFromHueSaturationBrightness({
                    brightness: Math.min(100, current.brightness + movementStep)
                });
                break;
            case 'ArrowDown':
                keyboardEvent.preventDefault();
                updateColorFromHueSaturationBrightness({
                    brightness: Math.max(0, current.brightness - movementStep)
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
            ref={areaReference}
            class="ui-color-picker-sb"
            style={{ 'background-color': `hsl(${hueSaturationBrightness().hue}, 100%, 50%)` }}
            onMouseDown={handleMouseDown}
            onKeyDown={handleKeyDown}
            tabindex={0}
            role="slider"
            aria-label="Saturation and brightness"
            aria-valuetext={`Saturation ${Math.round(hueSaturationBrightness().saturation)}%, Brightness ${Math.round(hueSaturationBrightness().brightness)}%`}
        >
            <div class="ui-color-picker-sb-white" />
            <div class="ui-color-picker-sb-black" />
            <div
                class="ui-color-picker-thumb"
                style={{
                    left: `${hueSaturationBrightness().saturation}%`,
                    top: `${100 - hueSaturationBrightness().brightness}%`,
                    display: activeColor() === 'transparent' ? 'none' : 'block'
                }}
            />
        </div>
    );
};
