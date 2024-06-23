import { ArrowUpTrayIcon } from "@heroicons/react/24/outline";
import { Fragment, useRef } from "react";
import { useDispatch } from "react-redux";

import { useLocalFiles } from "../../queries/file";
import { storeUrl } from "../../slices/UrlSlice";

export default function BottomUpload() {
  const modalRef = useRef<HTMLDialogElement>(null);
  const dispatch = useDispatch();

  const { data } = useLocalFiles();

  const handlePictureClick = (url: string) => {
    dispatch(storeUrl(url));
    modalRef?.current?.close();
  };

  return (
    <>
      <div
        onClick={() => modalRef?.current?.showModal()}
        className="mr-4 rounded-lg p-1 hover:bg-slate-50 hover:bg-opacity-10 active:bg-slate-700"
      >
        <ArrowUpTrayIcon width={24} height={24} />
      </div>
      <dialog ref={modalRef} className="modal">
        <div className="modal-box max-w-screen-xl">
          {(data?.pages ?? []).map((page, i) => (
            <div key={i} className="flex">
              <Fragment>
                {page.map((file) => (
                  <div key={file.id}>
                    <img
                      src={file.url}
                      alt={file.alt ?? undefined}
                      className="cursor-pointer rounded-lg border border-solid hover:shadow-lg"
                      onClick={() => handlePictureClick(file.url)}
                    />
                  </div>
                ))}
              </Fragment>
            </div>
          ))}
        </div>
        <form method="dialog" className="modal-backdrop">
          <button>close</button>
        </form>
      </dialog>
    </>
  );
}
