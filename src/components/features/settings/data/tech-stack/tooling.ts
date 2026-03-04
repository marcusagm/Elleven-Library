import { TechCategory } from './types';

export const frontendTooling: TechCategory = {
    title: 'Frontend Development & Tooling',
    items: [
        {
            name: 'ESLint & Prettier',
            description: 'Code linting and formatting.',
            url: 'https://eslint.org/'
        },
        {
            name: 'Vitest & Testing Library',
            description: 'Next generation testing framework.',
            url: 'https://vitest.dev/'
        },
        {
            name: 'Husky & lint-staged',
            description: 'Modern native git hooks.',
            url: 'https://typicode.github.io/husky/'
        },
        {
            name: 'JSDOM',
            description: 'A JS implementation of various web standards.',
            url: 'https://github.com/jsdom/jsdom'
        }
    ]
};
