import { useSetting } from "../../queries/setting";
import LoadingPage from "../Loading";

export default function LogOutIndexPage() {
  const { data: setting } = useSetting();

  if (setting == null) {
    return <LoadingPage />;
  }

  return <div>{setting.instanceDescription}</div>;
}
