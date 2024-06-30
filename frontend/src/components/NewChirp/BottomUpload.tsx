import { ArrowUpTrayIcon } from "@heroicons/react/24/outline";
import { useSetAtom } from "jotai";
import { Fragment, useRef } from "react";

import { useLocalFiles } from "../../queries/file";
import { pictureUrl } from "../../states/states";

export default function BottomUpload() {
  const modalRef = useRef<HTMLDialogElement>(null);
  const setUrl = useSetAtom(pictureUrl);

  const { data } = useLocalFiles();

  const handlePictureClick = (url: string) => {
    setUrl((el) => [...el, url]);

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
                {page.length !== 0 ? (
                  <div>
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
                  </div>
                ) : (
                  <span>no items</span>
                )}
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
