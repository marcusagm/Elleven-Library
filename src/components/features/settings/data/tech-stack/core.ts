import { TechCategory } from './types';

export const coreInfrastructure: TechCategory = {
    title: 'Core Infrastructure',
    items: [
        {
            name: 'Tauri v2',
            description: 'Secure desktop framework building bridges between Rust and JS.',
            url: 'https://tauri.app/'
        },
        {
            name: 'SolidJS',
            description: 'Performant, reactive UI framework with zero-overhead.',
            url: 'https://www.solidjs.com/'
        },
        {
            name: 'Rust',
            description: 'The safe, fast engine powering all native operations.',
            url: 'https://www.rust-lang.org/'
        },
        {
            name: 'Vite',
            description: 'Next-generation frontend tooling and build system.',
            url: 'https://vitejs.dev/'
        },
        {
            name: 'TypeScript',
            description: 'Strongly typed programming language that builds on JavaScript.',
            url: 'https://www.typescriptlang.org/'
        }
    ]
};
