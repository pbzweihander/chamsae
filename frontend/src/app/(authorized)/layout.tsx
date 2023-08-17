import MainContainer from "@/components/MainContainer";
import Nav from "@/components/Nav";

export default function RootLayout({
  children,
}: {
  children?: React.ReactNode;
}) {
  return (
    <div className="flex-1 w-full max-w-screen-2xl self-center grid grid-cols-[320px_1fr_320px]">
      <Nav />
      <MainContainer>
        {children}
      </MainContainer>
      <section className="border-l px-4 py-12">
        Additional
      </section>
    </div>
  );
}
