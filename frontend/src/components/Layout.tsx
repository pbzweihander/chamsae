import { Outlet } from "react-router-dom";

import LeftNav from "./LeftNav";
import RightNav from "./RightNav";

export default function Layout() {
  return (
    <div className="flex h-screen w-full justify-center">
      <LeftNav />
      <div className="mx-5 h-full w-1/2">
        <Outlet />
      </div>
      <RightNav />
    </div>
  );
}
