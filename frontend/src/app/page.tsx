import getAccessKey from "@/lib/api/getAccessKey";
import { redirect } from "next/navigation";

export default async function Main() {
  const accessKey = await getAccessKey();
  if (accessKey != null) {
    redirect("/feed");
  } else {
    redirect("/login");
  }
}
