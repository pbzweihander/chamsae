import { PlusIcon } from "@heroicons/react/24/outline";
import { useEffect, useRef, useState } from "react";
import { SubmitHandler, useForm } from "react-hook-form";

import { useLocalFileUploadMutation } from "../queries/file";

// import { PostFileQuery } from "../queries/file";

interface UploadForm {
  files: FileList;
  alt?: string;
}

export default function RightNavUpload() {
  const modalRef = useRef<HTMLDialogElement>(null);
  const [imagePreview, setImagePreview] = useState<string>("");
  const { register, handleSubmit, reset, watch } = useForm<UploadForm>();
  const {
    mutate: upload,
    isLoading,
    error,
  } = useLocalFileUploadMutation(() => {
    reset();
    modalRef.current?.close();
  });
  const image = watch("files");

  const onSubmit: SubmitHandler<UploadForm> = (data) => {
    if (!data.files) {
      return;
    }
    const file = data.files[0];

    upload({ file, mediaType: file.type, alt: data.alt });
  };

  useEffect(() => {
    if (image && image.length > 0) {
      const file = image[0];
      setImagePreview(URL.createObjectURL(file));
    } else {
      setImagePreview("");
    }
  }, [image]);

  return (
    <>
      <button
        className="flex items-center"
        onClick={() => {
          modalRef.current?.showModal();
        }}
      >
        <PlusIcon className="mr-3 inline h-10 w-10" />
        <span className="text-lg">Upload</span>
      </button>
      <dialog ref={modalRef} className="modal">
        <div className="modal-box">
          <h2 className="mb-4 text-lg font-bold">Upload</h2>
          <form className="form-control" onSubmit={handleSubmit(onSubmit)}>
            <input
              type="file"
              className="file-input file-input-bordered mb-4 w-full"
              required
              {...register("files")}
            />
            <div className="mb-4 flex h-[24rem] w-full items-center justify-center border border-solid">
              {imagePreview !== "" ? (
                <img src={imagePreview} alt="uploaded image" />
              ) : (
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="48"
                  height="48"
                  viewBox="0 0 24 24"
                >
                  <path
                    fill="#e6e6e6"
                    d="M14 9l-2.519 4-2.481-1.96-5 6.96h16l-6-9zm8-5v16h-20v-16h20zm2-2h-24v20h24v-20zm-20 6c0-1.104.896-2 2-2s2 .896 2 2c0 1.105-.896 2-2 2s-2-.895-2-2z"
                  />
                </svg>
              )}
            </div>
            <textarea
              className="textarea textarea-bordered mb-4 w-full"
              placeholder="Alt text..."
              {...register("alt")}
            />
            <input
              type="submit"
              className="btn btn-primary"
              value="Upload"
              disabled={isLoading}
            />
            {error && <div className="mt-5 text-error">{error.message}</div>}
          </form>
        </div>
        <form method="dialog" className="modal-backdrop">
          <button>close</button>
        </form>
      </dialog>
    </>
  );
}
