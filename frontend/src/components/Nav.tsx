import Link from "next/link";

export default function Nav() {
  return (
    <nav className="border-r px-4 py-12 flex flex-col">
      <Link href="/feed">Feed</Link>
      <Link href="/notifications">Notifications</Link>
      <Link href="/settings">Settings</Link>
    </nav>
  );
}
