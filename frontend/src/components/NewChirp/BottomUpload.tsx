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
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
          className="h-6 w-6"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5m-13.5-9L12 3m0 0 4.5 4.5M12 3v13.5"
          />
        </svg>
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
