import { Component, For, Show } from 'solid-js';
import { cn } from '../../../lib/utils';
import { Slider } from '../Slider';
import { useAudioContext } from './AudioPlayerContext';

export const AudioWaveform: Component = () => {
    const { audioRef, duration, currentTime, buffered, displayWaveform } = useAudioContext();

    return (
        <div class="ui-audio-seekbar-container">
            <div class="ui-audio-waveform">
                <Show
                    when={displayWaveform().length > 0}
                    fallback={
                        <div
                            class="ui-audio-waveform-bar"
                            style={{ width: '100%', height: '2px' }}
                        />
                    }
                >
                    <For each={displayWaveform()}>
                        {(waveformValue, stepIndex) => {
                            const isPlayed = () => {
                                const totalDuration = duration();
                                if (totalDuration === 0) return false;
                                const playbackPercentage = (currentTime() / totalDuration) * 100;
                                const pointPercentage =
                                    (stepIndex() / (displayWaveform().length || 1)) * 100;
                                return pointPercentage <= playbackPercentage;
                            };

                            const isBuffered = () => {
                                const totalDuration = duration();
                                if (totalDuration === 0) return true;
                                return (
                                    stepIndex() / (displayWaveform().length || 1) <=
                                    buffered() / totalDuration
                                );
                            };

                            return (
                                <div
                                    class={cn('ui-audio-waveform-bar', isPlayed() && 'is-played')}
                                    style={{
                                        height: `${Math.max(15, waveformValue * 100)}%`,
                                        opacity: isBuffered() ? 1 : 0.3
                                    }}
                                />
                            );
                        }}
                    </For>
                </Show>
            </div>
            <Slider
                minimumValue={0}
                maximumValue={duration()}
                stepValue={0.1}
                showTicks={false}
                value={currentTime()}
                onValueChange={newSeekTime => {
                    const audioElement = audioRef();
                    if (audioElement) audioElement.currentTime = newSeekTime;
                }}
                class="ui-audio-seekbar-slider"
            />
        </div>
    );
};
