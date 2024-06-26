import { useIsAuthed } from "../queries/auth";
import { useSetting } from "../queries/setting";
import RightNavGallery from "./RightNavGallery";
import RightNavSetting from "./RightNavSetting";
import RightNavUpload from "./RightNavUpload";

export default function RightNav() {
  const { data: setting } = useSetting();
  const isAuthed = useIsAuthed() ?? false;

  return (
    <div className="w-50 h-full p-4">
      {isAuthed && (
        <>
          <div className="mb-4">
            {setting && <RightNavSetting setting={setting} />}
          </div>
          <div className="mb-4">
            <RightNavUpload />
          </div>
          <div>
            <RightNavGallery />
          </div>
        </>
      )}
    </div>
  );
}
