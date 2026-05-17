// Ambient declaration for plain `.css` side-effect imports (e.g.
// `import "./globals.css"` in app/layout.tsx).
//
// Next.js ships declarations for `*.module.css` in node_modules/next/types,
// but NOT for plain `*.css`. Pre-TS 6, this didn't matter because TSC
// skipped side-effect import checks by default. TS 6.0 flipped
// `noUncheckedSideEffectImports` on by default, so without this declaration
// the import errors with TS2882.
//
// Alternative: set `"noUncheckedSideEffectImports": false` in tsconfig.json
// (loses the typo-protection benefit for other side-effect imports).
declare module "*.css";
