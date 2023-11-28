"use server";

import { cookies } from "next/headers";
import { throwError } from "../dto";
import { apiUrl } from ".";
import getAccessKeyOrRedirect from "./getAccessKeyOrRedirect";

export default async function logout() {
  const accessKey = await getAccessKeyOrRedirect();
  const resp = await fetch(apiUrl("/api/auth/logout"), {
    method: "POST",
    headers: {
      "authorization": `Bearer ${accessKey}`,
    },
    cache: "no-cache",
  });
  if (!resp.ok) {
    await throwError(resp);
  }
  cookies().delete("accessKey");
}
