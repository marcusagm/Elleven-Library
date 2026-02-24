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
    plugins: ['solid', '@typescript-eslint', 'prettier'],
    rules: {
        /*
         * Format & basic style
         */
        semi: ['error', 'always'], // require semicolons
        indent: ['error', 4, { SwitchCase: 1 }], // 4 spaces
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
        // 'no-unused-vars': [
        //     'warn',
        //     {
        //         argsIgnorePattern: '^_',
        //         ignoreRestSiblings: true,
        //         caughtErrors: 'none'
        //     }
        // ], // warn unused vars

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
        'array-bracket-spacing': ['error', 'never']
        // 'space-before-function-paren': ['error', 'never']
    }
};
