import { createContext, useContext, JSX } from 'solid-js';
import { useAudioPlayer } from './useAudioPlayer';
import { AudioPlayerProps } from './types';

type AudioContextType = ReturnType<typeof useAudioPlayer> & { props: AudioPlayerProps };

const AudioPlayerContext = createContext<AudioContextType>();

export const AudioProvider = (props: AudioPlayerProps & { children: JSX.Element }) => {
    const logic = useAudioPlayer(props);

    return (
        <AudioPlayerContext.Provider
            value={{
                ...logic,
                get props() {
                    return props;
                }
            }}
        >
            {props.children}
        </AudioPlayerContext.Provider>
    );
};

export const useAudioContext = () => {
    const context = useContext(AudioPlayerContext);
    if (!context) throw new Error('useAudioContext must be used within an AudioProvider');
    return context;
};
