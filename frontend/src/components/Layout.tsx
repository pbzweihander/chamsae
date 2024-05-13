import { Outlet } from "react-router-dom";

import LeftNav from "./LeftNav";
import RightNav from "./RightNav";

export default function Layout() {
  return (
    <div className="flex h-screen w-full justify-center bg-base-200">
      <LeftNav />
      <div className="mx-5 h-full w-1/2 bg-base-100">
        <Outlet />
      </div>
      <RightNav />
    </div>
  );
}
