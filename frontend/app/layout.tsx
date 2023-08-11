import "./globals.css";

import localFont from "next/font/local";
import Image from "next/image";
import Link from "next/link";

import logo from "./logo.svg";

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
      <body className={pretendard.className}>
        <Layout>
          <Sidebar>
            <Logo />
            <Link href="/">
              Home
            </Link>
            <Link href="/about">
              About
            </Link>
          </Sidebar>
          <Content>{children}</Content>
        </Layout>
      </body>
    </html>
  );
}

function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        display: "flex",
        maxWidth: 900,
        margin: "auto",
      }}
    >
      {children}
    </div>
  );
}

function Sidebar({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        padding: 20,
        flexShrink: 0,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        lineHeight: "1.8em",
      }}
    >
      {children}
    </div>
  );
}

function Content({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        padding: 20,
        paddingBottom: 50,
        borderLeft: "2px solid #eee",
        minHeight: "100vh",
      }}
    >
      {children}
    </div>
  );
}

function Logo() {
  return (
    <div
      style={{
        marginTop: 20,
        marginBottom: 10,
      }}
    >
      <a href="/">
        <Image src={logo} width="64" height="64" alt="logo" />
      </a>
    </div>
  );
}
