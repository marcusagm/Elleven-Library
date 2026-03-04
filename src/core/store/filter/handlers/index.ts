/**
 * Core Filter Domain Logic Dictionary
 *
 * @module CoreCriterionHandlerRegistry
 * @description
 * The index file serves as the centralized Barrel Export unifying all localized CoreCriterionHandlers.
 * Ensures the Single Source of Truth architecture isolating generic validation logic, parsing, and
 * formatting computations cleanly per parameter type constraint avoiding "God-Files".
 *
 * @example
 * ```ts
 * import { coreCriterionHandlerRegistry } from '@/core/store/filter/handlers';
 *
 * const specificHandler = coreCriterionHandlerRegistry["number"];
 * const { finalValue } = specificHandler.process("5", "10", "between");
 * ```
 */

import { colorHandler } from './colorHandler';
import { dateHandler } from './dateHandler';
import { folderHandler } from './folderHandler';
import { numberHandler } from './numberHandler';
import { ratingHandler } from './ratingHandler';
import { selectHandler } from './selectHandler';
import { sizeHandler } from './sizeHandler';
import { tagsHandler } from './tagsHandler';
import { textHandler } from './textHandler';
import { CoreCriterionHandler } from './types';

export * from './types';
export * from './utils';
export { textHandler };

/**
 * Registry mapping generic field identifiers to their rigid mathematical business handler.
 *
 * @type {Record<string, CoreCriterionHandler>}
 */
export const coreCriterionHandlerRegistry: Record<string, CoreCriterionHandler> = {
    color: colorHandler,
    date: dateHandler,
    folder: folderHandler,
    number: numberHandler,
    rating: ratingHandler,
    select: selectHandler,
    size: sizeHandler,
    tags: tagsHandler,
    text: textHandler
};
