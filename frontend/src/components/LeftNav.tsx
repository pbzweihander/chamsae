import { useIsAuthed } from "../queries/auth";
import LeftNavLogin from "./LeftNavLogin";
import LeftNavProfile from "./LeftNavProfile";

export default function LeftNav() {
  const isAuthed = useIsAuthed() ?? false;
  return (
    <div className="flex h-full w-64 flex-col p-4">
      <div className="flex-grow" />
      {isAuthed ? <LeftNavProfile /> : <LeftNavLogin />}
    </div>
  );
}
