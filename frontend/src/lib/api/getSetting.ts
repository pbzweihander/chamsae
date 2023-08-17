"use server";

import { Setting, throwError } from "@/lib/dto";
import { redirect } from "next/navigation";
import * as z from "zod";
import { apiUrl } from ".";

export default async function getSetting(): Promise<z.infer<typeof Setting>> {
  const resp = await fetch(apiUrl("/api/setting"));
  if (!resp.ok) {
    if (resp.status == 404) {
      redirect("/initialize");
    } else {
      await throwError(resp);
    }
  }

  return Setting.parse(await resp.json());
}
