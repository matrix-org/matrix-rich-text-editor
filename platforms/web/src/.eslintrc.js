module.exports = {
    'plugins': ['matrix-org'],
    'extends': ['plugin:matrix-org/typescript', 'plugin:matrix-org/react', 'plugin:matrix-org/a11y'],
    'env': {
        'browser': true,
        'node': true,
    },
    'settings': {
        'react': {
            'version': 'detect',
        },
    },
    'rules': {
        'new-cap': 'off',
        '@typescript-eslint/naming-convention': [
            'error',
            {
                'selector': ['variable', 'function'],
                'modifiers': ['private'],
                'format': ['camelCase'],
                'leadingUnderscore': 'allow',
            },
        ],
    },
};
