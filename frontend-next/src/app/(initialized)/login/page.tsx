import login from "@/lib/api/login";
import { redirect } from "next/navigation";

async function loginAction(formData: FormData) {
  "use server";

  const password = formData.get("password");

  if (password == null) {
    return;
  }

  await login(password.toString());
  redirect("/feed");
}

export default function Login() {
  return (
    <form action={loginAction}>
      <label className="mr-2">Password:</label>
      <input type="password" name="password" className="border-2 mr-2" />
      <input type="submit" className="bg-slate-200 px-2" value="Login" />
    </form>
  );
}
