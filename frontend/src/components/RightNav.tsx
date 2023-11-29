import { useSetting } from "../queries/setting";
import RightNavSetting from "./RightNavSetting";

export default function RightNav() {
  const { data: setting } = useSetting();

  return (
    <div className="h-full p-4">
      {setting && <RightNavSetting setting={setting} />}
    </div>
  );
}
