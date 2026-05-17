// Type declarations for CSS side-effect imports (used in app/layout.tsx)
declare module "*.css" {
  const content: Record<string, string>;
  export default content;
}
