import initializeInstance from "@/lib/api/initializeInstance";
import { redirect } from "next/navigation";

async function initializeAction(formData: FormData) {
  "use server";

  const instanceName = formData.get("instanceName");
  const userHandle = formData.get("userHandle");
  const userPassword = formData.get("userPassword");

  if (
    instanceName?.toString() == null || userHandle?.toString() == null
    || userPassword?.toString() == null
  ) {
    return;
  }

  await initializeInstance(instanceName.toString(), userHandle.toString(), userPassword.toString());
  redirect("/");
}

export default function Initialize() {
  return (
    <form action={initializeAction}>
      <div>
        <label className="mr-2">Instance name:</label>
        <input type="text" name="instanceName" required className="border-2 mr-2" />
      </div>
      <div>
        <label className="mr-2">User handle:</label>
        <input type="text" name="userHandle" required autoComplete="username" className="border-2 mr-2" placeholder="admin" />
      </div>
      <div>
        <label className="mr-2">User password:</label>
        <input type="password" name="userPassword" required autoComplete="new-password" className="border-2 mr-2" />
      </div>
      <input type="submit" className="bg-slate-200 px-2" value="Initialize" />
    </form>
  );
}
