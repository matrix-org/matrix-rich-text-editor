module.exports = {
    plugins: [
        "matrix-org",
    ],
    extends: [
     //   "plugin:matrix-org/babel",
        "plugin:matrix-org/react",
        "plugin:matrix-org/a11y",
    ],
    env: {
        browser: true,
        node: true,
    },
    globals: {
        LANGUAGES_FILE: "readonly",
    },
    rules: {
        // Things we do that break the ideal style
        "no-constant-condition": "off",
        "prefer-promise-reject-errors": "off",
        "no-async-promise-executor": "off",
        "quotes": "off",
        "no-extra-boolean-cast": "off",

        // Bind or arrow functions in props causes performance issues (but we
        // currently use them in some places).
        // It's disabled here, but we should using it sparingly.
        "react/jsx-no-bind": "off",
        "react/jsx-key": ["error"],

    //    "matrix-org/require-copyright-header": "error",
    },
    overrides: [
        {
            files: [
                "src/**/*.{ts,tsx}",
                "test/**/*.{ts,tsx}",
                "cypress/**/*.ts",
            ],
            extends: [
                "plugin:matrix-org/typescript",
                "plugin:matrix-org/react",
            ],
            rules: {
                // Things we do that break the ideal style
                "prefer-promise-reject-errors": "off",
                "quotes": "off",
                "no-extra-boolean-cast": "off",

                // Remove Babel things manually due to override limitations
                "@babel/no-invalid-this": ["off"],

                // We're okay being explicit at the moment
                "@typescript-eslint/no-empty-interface": "off",
                // We disable this while we're transitioning
                "@typescript-eslint/no-explicit-any": "off",
                // We'd rather not do this but we do
                "@typescript-eslint/ban-ts-comment": "off",
                // We're okay with assertion errors when we ask for them
                "@typescript-eslint/no-non-null-assertion": "off",

                // The non-TypeScript rule produces false positives
                "func-call-spacing": "off",
                "@typescript-eslint/func-call-spacing": ["error"],
            },
        },
    ],
    settings: {
        react: {
            version: "detect",
        },
    },
};