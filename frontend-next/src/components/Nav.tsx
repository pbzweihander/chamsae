"use client";

import logout from "@/lib/api/logout";
import Link from "next/link";
import { useRouter } from "next/navigation";

export default function Nav() {
  const router = useRouter();

  return (
    <nav className="border-r px-4 py-12 flex flex-col">
      <Link href="/feed">Feed</Link>
      <Link href="/notifications">Notifications</Link>
      <Link href="/settings">Settings</Link>
      <button
        onClick={async () => {
          await logout();
          router.push("/");
          router.refresh();
        }}
      >
        Logout
      </button>
    </nav>
  );
}
