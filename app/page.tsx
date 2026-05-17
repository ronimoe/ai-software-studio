import { ThemeToggle } from "@/components/layout/theme-toggle";

export default function HomePage() {
  return (
    <main className="flex min-h-screen items-center justify-center p-8">
      <div className="flex flex-col items-center gap-4">
        <h1 className="text-3xl font-semibold">AI Software Studio</h1>
        <p className="text-muted-foreground">Scaffold checkpoint — panels coming next.</p>
        <ThemeToggle />
      </div>
    </main>
  );
}
