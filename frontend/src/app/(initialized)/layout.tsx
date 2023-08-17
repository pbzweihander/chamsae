import getSetting from "@/lib/api/getSetting";

export default async function RootLayout({
  children,
}: {
  children?: React.ReactNode;
}) {
  await getSetting();
  return <>{children}</>;
}
