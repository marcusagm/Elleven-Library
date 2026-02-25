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
 * Parses an ISO standard date string (YYYY-MM-DD) to a Date object.
 *
 * @param {string} iso - The ISO standard date string.
 * @returns {Date | null} The corresponding date object, or null if invalid.
 */
export const fromISO = (iso: string): Date | null => {
    if (!iso || typeof iso !== 'string') return null;
    const parts = iso.split('-');
    if (parts.length === 3) {
        const year = parseInt(parts[0], 10);
        const month = parseInt(parts[1], 10) - 1;
        const day = parseInt(parts[2], 10);
        const date = new Date(year, month, day);
        return isNaN(date.getTime()) ? null : date;
    }
    return null;
};

/**
 * Formats a Date object to a display-ready string (DD/MM/YYYY).
 *
 * @param {Date} date - The date object to format.
 * @returns {string} The formatted display date string.
 */
export const formatDateToDisplay = (date: Date): string => {
    if (!(date instanceof Date) || isNaN(date.getTime())) return '';
    const day = date.getDate().toString().padStart(2, '0');
    const month = (date.getMonth() + 1).toString().padStart(2, '0');
    const year = date.getFullYear().toString();
    return `${day}/${month}/${year}`;
};

/**
 * Parses a display date string (DD/MM/YYYY) to a Date object.
 *
 * @param {string} displayString - The string in DD/MM/YYYY format.
 * @returns {Date | null} The corresponding date object, or null if invalid.
 */
export const parseDisplayDate = (displayString: string): Date | null => {
    if (!displayString || displayString.length < 10) return null;
    const parts = displayString.split('/');
    if (parts.length !== 3) return null;

    const day = parseInt(parts[0], 10);
    const month = parseInt(parts[1], 10) - 1; // Month is 0-indexed
    const year = parseInt(parts[2], 10);

    const date = new Date(year, month, day);

    // Validation to ensure it's a real date (e.g. not 31/02/2021)
    if (date.getFullYear() === year && date.getMonth() === month && date.getDate() === day) {
        return date;
    }

    return null;
};

/**
 * Formats a date (ISO string or Date object) to a localized display format (DD/MM/YYYY).
 *
 * @param {string | Date} val - The date to format.
 * @returns {string} The formatted display date string.
 */
export const formatToDisplay = (val: string | Date): string => {
    if (val instanceof Date) {
        return formatDateToDisplay(val);
    }
    if (typeof val !== 'string') return '';

    const date = fromISO(val);
    return date ? formatDateToDisplay(date) : val;
};
