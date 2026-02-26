/**
 * Accordion component
 *
 * @module Accordion
 * @description
 * The Accordion component is a compound component system for creating vertically stacked expandable content panels.
 * Built with Solid.js for high performance and full accessibility compliance.
 *
 * @example
 * ```tsx
 * import { Accordion } from '@/components/ui';
 *
 * <Accordion
 *   type="multiple"
 *   value={['item-1']}
 *   defaultValue={['item-1']}
 *   onValueChange={(value) => console.log(value)}
 *   collapsible={true}
 *   disabled={false}
 *   class="custom-class"
 * >
 *   <AccordionItem value="unique-id">
 *     <AccordionTrigger>Toggle Me</AccordionTrigger>
 *     <AccordionContent>Visible when expanded</AccordionContent>
 *   </AccordionItem>
 * </Accordion>
 * ```
 */

export { Accordion } from './AccordionRoot';
export { AccordionItem } from './AccordionItem';
export { AccordionTrigger, AccordionHeader, AccordionChevron } from './AccordionTrigger';
export { AccordionContent } from './AccordionContent';
export * from './types';
export * from './useAccordion';
