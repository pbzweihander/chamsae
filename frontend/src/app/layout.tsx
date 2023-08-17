import "./globals.css";

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
        {children}
      </body>
    </html>
  );
}
