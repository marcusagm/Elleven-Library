module.exports = {
    env: {
        browser: true,
        es2021: true,
        node: true
    },
    extends: [
        'eslint:recommended',
        'plugin:solid/typescript',
        'plugin:@typescript-eslint/recommended',
        'plugin:prettier/recommended'
    ],
    parser: '@typescript-eslint/parser',
    parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module'
    },
    plugins: ['solid', '@typescript-eslint', 'prettier', 'import'],
    settings: {
        'import/resolver': {
            typescript: {
                alwaysTryTypes: true,
                project: './tsconfig.json'
            }
        }
    },
    rules: {
        /*
         * Format & basic style
         */
        semi: ['error', 'always'], // require semicolons
        quotes: ['error', 'single', { avoidEscape: true }], // prefer single quotes
        'comma-dangle': ['error', 'never'], // no trailing commas
        'no-trailing-spaces': 'error', // no trailing spaces
        'eol-last': ['error', 'always'], // newline at EOF

        /*
         * Variables & scope
         */
        'no-var': 'error', // disallow var
        'prefer-const': ['error', { destructuring: 'all' }], // prefer const
        'no-global-assign': 'error', // prevent assignment to globals
        'no-implicit-globals': 'error', // prevent implicit globals (browser)

        /*
         * Equality & coercion
         */
        eqeqeq: ['error', 'always'], // enforce strict equality (===)
        'no-implicit-coercion': ['error', { allow: ['!!'] }], // avoid implicit coercion

        /*
         * Prettier
         */
        'prettier/prettier': 'warn',

        /*
         * TypeScript
         */
        '@typescript-eslint/no-explicit-any': 'warn',

        /*
         * Complexity
         */
        complexity: ['warn', 10],
        'max-lines': ['warn', { max: 300, skipBlankLines: true, skipComments: true }],

        /*
         * Console
         */
        'no-console': ['warn', { allow: ['warn', 'error', 'info'] }],
        'no-undef': 'off',

        /*
         * Imports & modules
         */
        'no-duplicate-imports': 'error', // disallow duplicate imports

        /*
         * Stylistic preferences (Prettier will override formatting rules)
         */
        'object-curly-spacing': ['error', 'always'],
        'array-bracket-spacing': ['error', 'never'],

        /*
         * Architectural Guardrails (Sprint 0)
         */
        'import/no-cycle': 'error',
        'import/no-restricted-paths': [
            'error',
            {
                zones: [
                    {
                        target: '**/components/**',
                        from: '**/core/tauri/**',
                        message:
                            'UI components must not call Tauri services directly. Use an Action in the Core layer instead.'
                    },
                    {
                        target: '**/core/store/**',
                        from: '**/components/ui/**',
                        message:
                            'Stores must not use UI elements/toasts directly. Return an error or use Domain Events.'
                    }
                ]
            }
        ]
    }
};
