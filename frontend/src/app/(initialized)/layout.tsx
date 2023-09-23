import getSetting from "@/lib/api/getSetting";

export const dynamic = "force-dynamic";

export default async function RootLayout({
  children,
}: {
  children?: React.ReactNode;
}) {
  await getSetting();
  return <>{children}</>;
}
