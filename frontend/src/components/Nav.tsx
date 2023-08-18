"use client";

import logout from "@/lib/api/logout";
import Link from "next/link";
import { redirect } from "next/navigation";

export default function Nav() {
  return (
    <nav className="border-r px-4 py-12 flex flex-col">
      <Link href="/feed">Feed</Link>
      <Link href="/notifications">Notifications</Link>
      <Link href="/settings">Settings</Link>
      <button
        onClick={async () => {
          await logout();
          redirect("/");
        }}
      >
        Logout
      </button>
    </nav>
  );
}
