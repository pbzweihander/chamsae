import "./globals.css";

import MainContainer from "@/components/MainContainer";
import Nav from "@/components/Nav";
import localFont from "next/font/local";

const pretendard = localFont({
  src: "./pretendard.woff2",
  display: "swap",
});

export const metadata = {
  title: {
    template: "%s | Chamsae",
    default: "Chamsae",
  },
  formatDetection: {
    address: false,
    telephone: false,
  },
};

export default function RootLayout({
  children,
}: {
  children?: React.ReactNode;
}) {
  return (
    <html>
      <body className={"flex flex-col items-stretch min-h-screen " + pretendard.className}>
        <div className="flex-1 w-full max-w-screen-2xl self-center grid grid-cols-[320px_1fr_320px]">
          <Nav />
          <MainContainer>
            {children}
          </MainContainer>
          <section className="border-l px-4 py-12">
            Additional
          </section>
        </div>
      </body>
    </html>
  );
}
