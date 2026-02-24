import { createContext, useContext, JSX } from 'solid-js';
import { useVideoPlayer } from './useVideoPlayer';
import { VideoPlayerProps } from './types';

type VideoContextValue = ReturnType<typeof useVideoPlayer> & {
    props: VideoPlayerProps;
};

const VideoPlayerContext = createContext<VideoContextValue>();

/**
 * Context Provider for the Video Player.
 * Makes the initialized player state and handlers available down the component tree.
 *
 * @param props - Children components and underlying VideoPlayerProps
 * @returns Context Provider node
 */
export const VideoProvider = (props: VideoPlayerProps & { children?: JSX.Element }) => {
    const logic = useVideoPlayer(props);

    return (
        <VideoPlayerContext.Provider
            value={
                {
                    ...logic,
                    get props() {
                        return props;
                    }
                } as unknown as VideoContextValue
            }
        >
            {props.children}
        </VideoPlayerContext.Provider>
    );
};

/**
 * Consumes the `VideoPlayerContext` throwing an error if used outside a provider.
 *
 * @returns The fully constructed context from `useVideoPlayer`
 */
export const useVideoContext = () => {
    const context = useContext(VideoPlayerContext);
    if (!context) {
        throw new Error('useVideoContext must be used within VideoProvider');
    }
    return context;
};
