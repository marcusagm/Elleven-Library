/**
 * Formats a file size in bytes to a human-readable string.
 *
 * @param {number} bytes - The size of the file in bytes.
 * @returns {string} The formatted file size.
 *
 * @example
 * const readableSize = formatFileSize(1024); // Returns "1 KB"
 */
export function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Formats a date string or object to a localized alphanumeric string.
 *
 * @param {string | Date} dateStr - The date to format.
 * @returns {string} The formatted date.
 */
export function formatDate(dateStr: string | Date): string {
    if (!dateStr) return '-';
    const date = typeof dateStr === 'string' ? new Date(dateStr) : dateStr;
    return new Intl.DateTimeFormat(navigator.language, {
        year: 'numeric',
        month: 'short',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
    }).format(date);
}

/**
 * Formats a date to a short numeric representation.
 *
 * @param {string | Date} dateStr - The date to format.
 * @returns {string} The short formatted date.
 */
export function formatShortDate(dateStr: string | Date): string {
    if (!dateStr) return '-';
    const date = typeof dateStr === 'string' ? new Date(dateStr) : dateStr;
    return new Intl.DateTimeFormat(navigator.language, {
        year: '2-digit',
        month: '2-digit',
        day: '2-digit'
    }).format(date);
}

/**
 * Formats a Date object or string to an ISO standard date string (YYYY-MM-DD).
 *
 * @param {Date | string} val - The date to convert to ISO string.
 * @returns {string} The formatted ISO date string.
 */
export const formatToISO = (val: Date | string): string => {
    if (val instanceof Date) {
        const year = val.getFullYear();
        const month = (val.getMonth() + 1).toString().padStart(2, '0');
        const day = val.getDate().toString().padStart(2, '0');
        return `${year}-${month}-${day}`;
    }
    return String(val);
};

/**
 * Formats an ISO standard date string to a localized display format (DD/MM/YYYY).
 *
 * @param {string} iso - The ISO string to format.
 * @returns {string} The formatted display date string.
 */
export const formatToDisplay = (iso: string) => {
    if (!iso || typeof iso !== 'string') return iso;
    const parts = iso.split('-');
    if (parts.length === 3) {
        const d = new Date(parseInt(parts[0]), parseInt(parts[1]) - 1, parseInt(parts[2]));
        if (!isNaN(d.getTime())) {
            const day = d.getDate().toString().padStart(2, '0');
            const month = (d.getMonth() + 1).toString().padStart(2, '0');
            const year = d.getFullYear();
            return `${day}/${month}/${year}`;
        }
    }
    return iso;
};

/**
 * Prepares a Date object from an ISO standard date string.
 *
 * @param {string} iso - The ISO standard date string.
 * @returns {Date | null} The corresponding date object, or null if invalid.
 */
export const fromISO = (iso: string) => {
    if (!iso || typeof iso !== 'string') return null;
    const parts = iso.split('-');
    if (parts.length === 3) {
        const d = new Date(parseInt(parts[0]), parseInt(parts[1]) - 1, parseInt(parts[2]));
        return isNaN(d.getTime()) ? null : d;
    }
    return null;
};
