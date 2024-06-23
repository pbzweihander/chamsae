import { PlusIcon, PhotoIcon } from "@heroicons/react/24/outline";
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
                <PhotoIcon width={48} height={48} stroke="#e6e6e6" />
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
