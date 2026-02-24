/**
 * Formats a given time in seconds into a traditional HH:MM:SS or MM:SS format.
 *
 * @param seconds - The time duration in seconds.
 * @returns The formatted time string.
 *
 * @example
 * const formatted = formatTime(125); // "02:05"
 */
export const formatTime = (seconds: number) => {
    if (!Number.isFinite(seconds) || Number.isNaN(seconds)) return '--:--';
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    const parts = [m, s].map(v => v.toString().padStart(2, '0'));
    if (h > 0) parts.unshift(h.toString());
    return parts.join(':');
};
