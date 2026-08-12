export interface AudioPlayerProps {
    src: string;
    filePath?: string; // Original system path for waveform extraction
    assetId?: string; // Asset ID for trash-aware path resolution
    variant?: 'full' | 'compact';
    autoPlay?: boolean;
    title?: string;
    subtitle?: string;
    onEnded?: () => void;
    onError?: (error: string) => void;
    class?: string;
}
