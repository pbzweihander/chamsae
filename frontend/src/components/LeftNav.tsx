import { useIsAuthed } from "../queries/auth";
import { useSetting } from "../queries/setting";
import LeftNavLogin from "./LeftNavLogin";
import LeftNavProfile from "./LeftNavProfile";

export default function LeftNav() {
  const { data: setting } = useSetting();
  const isAuthed = useIsAuthed() ?? false;

  return (
    <div className="flex h-full w-64 flex-col p-4">
      <h1 className="text-xl font-bold">{setting?.instanceName}</h1>
      <div className="flex-grow" />
      {isAuthed ? <LeftNavProfile /> : <LeftNavLogin />}
    </div>
  );
}
