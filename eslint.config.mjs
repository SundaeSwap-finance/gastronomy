import { fixupPluginRules } from "@eslint/compat";
import js from "@eslint/js";
import { configs } from '@sundaeswap/eslint-config';
import tsParser from "@typescript-eslint/parser";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";

export default [
    ...configs,
    js.configs.recommended, {
    files: ['**/*.{js,jsx,mjs,cjs,ts,tsx}'],
    plugins: {
        react,
        "react-hooks": fixupPluginRules(reactHooks),
        "react-refresh": fixupPluginRules(reactRefresh),
    },

    languageOptions: {
        globals: {
            ...globals.browser,
            JSX: "readonly",
            React: "readonly"
        },

        parser: tsParser,
    },

    rules: {
        ...react.configs.recommended.rules,
        ...reactHooks.configs.recommended.rules,
        "react-refresh/only-export-components": ["warn", {
            allowConstantExport: true,
        }],
        "no-console": "off",
    },

    settings: {
        react: {
            version: 'detect'
        }
    }
}];