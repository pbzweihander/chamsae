"use server";

import { cookies } from "next/headers";
import { apiUrl } from ".";

export default async function getAccessKey(): Promise<string | undefined> {
  const tokenCookie = cookies().get("accessKey");
  if (tokenCookie == null) {
    return;
  }
  const token = tokenCookie.value;

  const resp = await fetch(apiUrl("/api/auth/check"), {
    headers: {
      "authorization": `Bearer ${token}`,
    },
  });
  if (!resp.ok) {
    return;
  }

  return token;
}
