"use server";

import { redirect } from "next/navigation";
import getAccessKey from "./getAccessKey";

export default async function getAccessKeyOrRedirect(): Promise<string> {
  const accessKey = await getAccessKey();
  if (accessKey == null) {
    redirect("/login");
  }
  return accessKey;
}
