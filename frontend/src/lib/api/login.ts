'use server'

import * as z from "zod";
import { cookies } from "next/headers";

import { apiUrl } from "@/lib/api";
import { Id, throwError } from "@/lib/dto";

const PostLoginResp = z.object({
  token: Id,
});

export default async function login(password: string) {
  const resp = await fetch(apiUrl("/api/auth/login"), {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ password }),
  });
  if (!resp.ok) {
    await throwError(resp);
  }
  const parsed = PostLoginResp.parse(await resp.json());
  cookies().set("accessKey", parsed.token, { secure: true });
}
