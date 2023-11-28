export default function MainContainer({ children }: { children: React.ReactNode }) {
  return (
    <main className="px-4 py-12">
      <div className="max-w-screen-md mx-auto">
        {children}
      </div>
    </main>
  );
}
