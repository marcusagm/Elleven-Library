import { Component, Show } from 'solid-js';
import { Slider } from '../Slider';
import { useVideoContext } from './VideoPlayerContext';
import { formatTime } from './utils';

/**
 * Displays the video seekbar UI including the buffered progress and preview tooltip.
 * Connects directly to `useVideoContext` internally.
 *
 * @returns Video seekbar component
 */
export const VideoSeekbar: Component = () => {
    const {
        duration,
        currentTime,
        buffered,
        handleSeek,
        setPreviewTime,
        setPreviewPos,
        previewTime,
        previewPos
    } = useVideoContext();

    /**
     * Calculates the preview time based on mouse position over the seekbar.
     *
     * @param mouseEvent - The native MouseEvent.
     */
    const handleSeekMouseMove = (mouseEvent: MouseEvent) => {
        const trackElement = mouseEvent.currentTarget as HTMLElement;
        const trackBoundingRect = trackElement.getBoundingClientRect();
        const positionRatio =
            (mouseEvent.clientX - trackBoundingRect.left) / trackBoundingRect.width;
        const calculatedTime = positionRatio * duration();
        setPreviewTime(calculatedTime);
        setPreviewPos(positionRatio * 100);
    };

    return (
        <div
            class="ui-video-seekbar-container"
            onMouseMove={handleSeekMouseMove}
            onMouseLeave={() => setPreviewTime(null)}
        >
            <Show when={previewTime() !== null}>
                <div class="ui-video-seekbar-preview" style={{ left: `${previewPos()}%` }}>
                    {formatTime(previewTime()!)}
                </div>
            </Show>
            <div class="ui-video-seekbar">
                <div
                    class="ui-video-seekbar-buffer"
                    style={{ width: `${(buffered() / duration()) * 100}%` }}
                />
                <Slider
                    minimumValue={0}
                    maximumValue={duration()}
                    stepValue={0.1}
                    showTicks={false}
                    value={currentTime()}
                    onValueChange={handleSeek}
                    class="ui-video-seekbar-slider"
                />
            </div>
        </div>
    );
};
