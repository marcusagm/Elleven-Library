import { describe, it, expect, vi } from 'vitest';
import { getVideoUrl, getAudioUrl } from './stream-utils';
vi.mock('../core/store/formatStore', () => {
    const strategyMap: Record<string, string> = {
        mp4: 'native',
        mov: 'native',
        mkv: 'hls',
        avi: 'hls',
        swf: 'linearHls',
        mpg: 'linearHls',
        mp3: 'native',
        wav: 'native',
        flac: 'audioTranscode',
        ogg: 'audioTranscode'
    };

    return {
        formatActions: {
            getPlaybackStrategy: vi.fn((ext: string) => strategyMap[ext.toLowerCase()] || 'native'),
            getMediaType: vi.fn()
        }
    };
});

describe('stream-utils URL generation', () => {
    describe('getVideoUrl', () => {
        it('should return asset:// URL for native MP4 videos', () => {
            const path = '/Movies/test_video.mp4';
            const assetId = '12345';
            const url = getVideoUrl(assetId, path, 'standard');
            expect(url).toBe(`asset://localhost/${assetId}`);
        });

        it('should return HLS URL for standard transcoding videos (MKV)', () => {
            const path = '/Movies/test_video.mkv';
            const assetId = '12345';
            const url = getVideoUrl(assetId, path, 'standard');
            expect(url).toContain('/playlist/%2FMovies%2Ftest_video.mkv');
        });

        it('should return linear HLS URL for live transcoding videos (SWF)', () => {
            const path = '/Movies/test_video.swf';
            const assetId = '12345';
            const url = getVideoUrl(assetId, path, 'high');
            expect(url).toContain('/hls-live/');
            expect(url).toContain('mode=live');
            expect(url).toContain('quality=high');
        });
    });

    describe('getAudioUrl', () => {
        it('should return audio:// URL for native audio', () => {
            const path = '/Music/test_audio.mp3';
            const url = getAudioUrl(path, 'standard');
            expect(url).toBe('audio://localhost/%2FMusic%2Ftest_audio.mp3');
        });

        it('should return audio-stream:// URL for transcoded audio', () => {
            const path = '/Music/test_audio.flac';
            const url = getAudioUrl(path, 'standard');
            expect(url).toContain('audio-stream://localhost/');
            expect(url).toContain('quality=standard');
        });
    });
});
