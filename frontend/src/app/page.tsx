import getAccessKey from "@/lib/api/getAccessKey";
import getSetting from "@/lib/api/getSetting";
import { redirect } from "next/navigation";

export const dynamic = "force-dynamic";

export default async function Main() {
  await getSetting();
  const accessKey = await getAccessKey();
  if (accessKey != null) {
    redirect("/feed");
  } else {
    redirect("/login");
  }
}
