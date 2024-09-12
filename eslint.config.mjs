import { fixupPluginRules } from "@eslint/compat";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";
import tsParser from "@typescript-eslint/parser";
import { configs } from '@sundaeswap/eslint-config';

export default [
    ...configs,
    {
    files: ['**/*.{js,jsx,mjs,cjs,ts,tsx}'],
    plugins: {
        react,
        "react-hooks": fixupPluginRules(reactHooks),
        "react-refresh": fixupPluginRules(reactRefresh),
    },

    languageOptions: {
        globals: {
            ...globals.browser,
        },

        parser: tsParser,
    },

    rules: {
        ...reactHooks.configs.recommended.rules,
        "react-refresh/only-export-components": ["warn", {
            allowConstantExport: true,
        }],
        "no-console": "off",
        "@typescript-eslint/naming-convention": "off",
    },
}];