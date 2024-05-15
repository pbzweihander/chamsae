import { PhotoIcon, PlusIcon, TrashIcon } from "@heroicons/react/24/outline";
import { Fragment, useRef } from "react";

import { useLocalFileDeleteMutation, useLocalFiles } from "../queries/file";

export default function RightNavGallery() {
  const modalRef = useRef<HTMLDialogElement>(null);

  const { data, fetchNextPage, hasNextPage, isFetchingNextPage, remove } =
    useLocalFiles();
  const { mutate: deleteFile, isLoading: isDeleteLoading } =
    useLocalFileDeleteMutation(() => {
      remove();
    });

  return (
    <>
      <button
        className="flex items-center"
        onClick={() => {
          modalRef?.current?.showModal();
        }}
      >
        <PhotoIcon className="mr-3 inline h-10 w-10" />
        <span className="text-lg">Gallery</span>
      </button>
      <dialog ref={modalRef} className="modal">
        <div className="modal-box max-w-screen-xl">
          <h2 className="mb-4 text-lg font-bold">Gallery</h2>
          <div className="grid grid-cols-3 gap-4">
            {(data?.pages ?? []).map((page, i) => (
              <Fragment key={i}>
                {page.map((file) => (
                  <div
                    key={file.id}
                    className="relative w-full overflow-y-hidden shadow-lg after:block after:pb-[100%] after:content-['']"
                  >
                    <img
                      className="absolute w-full"
                      src={file.url}
                      alt={file.alt ?? undefined}
                    />
                    <button
                      className="btn btn-circle btn-error btn-sm absolute right-4 top-4"
                      disabled={isDeleteLoading}
                      onClick={() => {
                        deleteFile(file.id);
                      }}
                    >
                      <TrashIcon className="h-5 w-5" />
                    </button>
                  </div>
                ))}
              </Fragment>
            ))}
          </div>
          <div className="mt-4 flex w-full justify-center">
            {isFetchingNextPage ? (
              <span className="loading loading-spinner loading-lg" />
            ) : (
              hasNextPage && (
                <button
                  onClick={() => {
                    fetchNextPage();
                  }}
                >
                  <PlusIcon className="h-10 w-10" />
                </button>
              )
            )}
          </div>
        </div>
        <form method="dialog" className="modal-backdrop">
          <button>close</button>
        </form>
      </dialog>
    </>
  );
}
