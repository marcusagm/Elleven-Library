import { TechCategory } from './types';
import { coreInfrastructure } from './core';
import { frontendDependencies } from './frontend';
import { frontendTooling } from './tooling';
import { backendEngine } from './backend';
import { externalTools } from './external';

export const TECH_STACK: TechCategory[] = [
    coreInfrastructure,
    frontendDependencies,
    frontendTooling,
    backendEngine,
    externalTools
];

export * from './types';
