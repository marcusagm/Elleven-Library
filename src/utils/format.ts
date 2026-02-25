/**
 * Formats a file size in bytes to a human-readable string (e.g., "1.5 MB").
 *
 * @param {number} bytes - The size of the file in bytes.
 * @returns {string} The formatted file size with its appropriate unit.
 *
 * @example
 * const readableSize = formatFileSize(1024); // Returns "1 KB"
 */
export function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const bytesPerKilobyte = 1024;
    const sizeUnitList = ['B', 'KB', 'MB', 'GB', 'TB'];
    const unitIndex = Math.floor(Math.log(bytes) / Math.log(bytesPerKilobyte));
    return (
        parseFloat((bytes / Math.pow(bytesPerKilobyte, unitIndex)).toFixed(2)) +
        ' ' +
        sizeUnitList[unitIndex]
    );
}

/**
 * Formats a date string or Date object to a localized alphanumeric string including time.
 *
 * @param {string | Date} dateSource - The date string or object to format.
 * @returns {string} The formatted date string in localized format, or "-" if input is empty.
 */
export function formatDate(dateSource: string | Date): string {
    if (!dateSource) return '-';
    const dateObject = typeof dateSource === 'string' ? new Date(dateSource) : dateSource;
    return new Intl.DateTimeFormat(navigator.language, {
        year: 'numeric',
        month: 'short',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
    }).format(dateObject);
}

/**
 * Formats a date to a short localized numeric representation (e.g., "DD/MM/YY").
 *
 * @param {string | Date} dateSource - The date source string or object to format.
 * @returns {string} The short formatted date string.
 */
export function formatShortDate(dateSource: string | Date): string {
    if (!dateSource) return '-';
    const dateObject = typeof dateSource === 'string' ? new Date(dateSource) : dateSource;
    return new Intl.DateTimeFormat(navigator.language, {
        year: '2-digit',
        month: '2-digit',
        day: '2-digit'
    }).format(dateObject);
}

/**
 * Formats a Date object or string to an ISO standard date string (YYYY-MM-DD).
 *
 * @param {Date | string} dateSource - The date to convert to ISO string.
 * @returns {string} The formatted ISO date string or the original string if conversion is not applicable.
 */
export const formatToISO = (dateSource: Date | string): string => {
    if (dateSource instanceof Date) {
        const year = dateSource.getFullYear();
        const month = (dateSource.getMonth() + 1).toString().padStart(2, '0');
        const day = dateSource.getDate().toString().padStart(2, '0');
        return `${year}-${month}-${day}`;
    }
    return String(dateSource);
};

/**
 * Parses an ISO standard date string (YYYY-MM-DD) to a Date object.
 *
 * @param {string} isoString - The ISO standard date string to parse.
 * @returns {Date | null} The corresponding Date object, or null if the string is invalid or cannot be parsed.
 */
export const fromISO = (isoString: string): Date | null => {
    if (!isoString || typeof isoString !== 'string') return null;
    const dateParts = isoString.split('-');
    if (dateParts.length === 3) {
        const year = parseInt(dateParts[0], 10);
        const month = parseInt(dateParts[1], 10) - 1;
        const day = parseInt(dateParts[2], 10);
        const dateObject = new Date(year, month, day);
        return isNaN(dateObject.getTime()) ? null : dateObject;
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
 * Formats a date source (ISO string or Date object) to a localized display format (DD/MM/YYYY).
 *
 * @param {string | Date} dateSource - The date source to format.
 * @returns {string} The formatted display date string, or an empty string if invalid.
 */
export const formatToDisplay = (dateSource: string | Date): string => {
    if (dateSource instanceof Date) {
        return formatDateToDisplay(dateSource);
    }
    if (typeof dateSource !== 'string') return '';

    const dateObject = fromISO(dateSource);
    return dateObject ? formatDateToDisplay(dateObject) : dateSource;
};
