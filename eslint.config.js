import tsPlugin from "@typescript-eslint/eslint-plugin";
import tsParser from "@typescript-eslint/parser";
import vuePlugin from "eslint-plugin-vue";
import vueParser from "vue-eslint-parser";

const sharedLanguageOptions = {
  ecmaVersion: "latest",
  sourceType: "module",
};

export default [
  {
    ignores: ["dist/**", "node_modules/**", "src-tauri/**", "coverage/**"],
  },
  ...vuePlugin.configs["flat/recommended"],
  {
    files: ["src/**/*.ts", "vite.config.ts"],
    languageOptions: {
      ...sharedLanguageOptions,
      parser: tsParser,
    },
    plugins: {
      "@typescript-eslint": tsPlugin,
    },
    rules: {
      ...tsPlugin.configs.recommended.rules,
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
  {
    files: ["src/**/*.vue"],
    languageOptions: {
      ...sharedLanguageOptions,
      parser: vueParser,
      parserOptions: {
        parser: tsParser,
        extraFileExtensions: [".vue"],
      },
    },
    plugins: {
      "@typescript-eslint": tsPlugin,
    },
    rules: {
      ...tsPlugin.configs.recommended.rules,
      "vue/multi-word-component-names": "off",
      "vue/require-default-prop": "off",
      "vue/max-attributes-per-line": ["error", { "singleline": 8, "multiline": 1 }],
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
];
