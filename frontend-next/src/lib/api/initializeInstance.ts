"use server";

import { throwError } from "../dto";
import { apiUrl } from ".";

export default async function(instanceName: string, userHandle: string, userPassword: string) {
  const resp = await fetch(apiUrl("/api/setting/initial"), {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({ instanceName, userHandle, userPassword }),
  });
  if (!resp.ok) {
    await throwError(resp);
  }
}
