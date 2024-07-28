import { TrashIcon, ArrowUpTrayIcon } from "@heroicons/react/24/outline";
import { useAtomValue, useSetAtom } from "jotai";
import { useEffect, useRef } from "react";
import { SubmitHandler, useForm } from "react-hook-form";
import z from "zod";

import { CreatePost } from "../dto";
import { useLocalFiles } from "../queries/file";
import { usePostNoteMutation } from "../queries/note";
import { pictureUrl } from "../states/states";

export default function BottomNewChirp() {
  const modalRef = useRef<HTMLDialogElement>(null);
  const setUrl = useSetAtom(pictureUrl);

  const { data: localFiles } = useLocalFiles();

  const handlePictureClick = (url: string) => {
    setUrl((el) => [...el, url]);

    modalRef?.current?.close();
  };

  const { register, handleSubmit, setValue, reset } =
    useForm<z.infer<typeof CreatePost>>();
  const {
    mutate: postNote,
    isLoading,
    error,
  } = usePostNoteMutation(() => {
    reset();
  });
  const pictureUrlArr = useAtomValue(pictureUrl);
  const deleteUrl = useSetAtom(pictureUrl);

  const onSubmit: SubmitHandler<z.infer<typeof CreatePost>> = (data) => {
    postNote(data);
    deletePicture();
  };

  const deletePicture = () => {
    deleteUrl(() => []);
  };

  useEffect(() => {
    const ulid: string[] = pictureUrlArr.map((el: string) =>
      el.substring(el.length - 26, el.length),
    );
    setValue("files", ulid);
  }, [pictureUrlArr, setValue]);

  return (
    <>
      <form
        className="chat chat-end absolute bottom-4 right-4"
        onSubmit={handleSubmit(onSubmit)}
      >
        <input type="hidden" value="public" {...register("visibility")} />
        <div className="chat-bubble chat-bubble-primary">
          <div className="flex items-center">
            <button
              onClick={() => modalRef?.current?.showModal()}
              className="btn btn-ghost btn-sm mr-2"
            >
              <ArrowUpTrayIcon width={24} height={24} />
            </button>
            <div>
              {pictureUrlArr.length > 0 && (
                <div>
                  {pictureUrlArr.map((value, i) => (
                    <div key={i}>
                      <input
                        type="hidden"
                        value={pictureUrlArr}
                        {...register("files")}
                      />
                      <img
                        src={value}
                        alt="pictureurl"
                        className="w-48 rounded-lg"
                      />
                      <button
                        type="button"
                        className="btn btn-circle btn-error btn-sm absolute right-4 top-4"
                        onClick={() => deletePicture()}
                      >
                        <TrashIcon className="h-5 w-5" />
                      </button>
                    </div>
                  ))}
                </div>
              )}
              <input
                type="text"
                className="input w-full bg-transparent"
                placeholder="Write something..."
                required
                {...register("text")}
              />
            </div>
          </div>
        </div>
        <div className="chat-footer">
          <input
            type="submit"
            className="btn btn-primary btn-sm mt-2"
            value="Chirp!"
            disabled={isLoading}
          />
        </div>
        {error && <div className="mt-5 text-error">{error.message}</div>}
      </form>
      <dialog ref={modalRef} className="modal">
        <div className="modal-box max-w-screen-xl">
          {(localFiles?.pages ?? []).map((page, i) => (
            <div key={i} className="flex">
              <>
                {page.length !== 0 ? (
                  <div className="grid grid-cols-3 gap-4">
                    {page.map((file) => (
                      <div key={file.id} className="h-48 w-48">
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
              </>
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
