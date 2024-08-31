import { Cog6ToothIcon } from "@heroicons/react/24/solid";
import { useRef } from "react";
import { SubmitHandler, useForm } from "react-hook-form";
import z from "zod";

import { Setting } from "../dto";
import { PutSettingReq, usePutSettingMutation } from "../queries/setting";

export default function RightNavSetting({
  setting,
}: {
  setting: z.infer<typeof Setting>;
}) {
  const modalRef = useRef<HTMLDialogElement>(null);
  const { register, handleSubmit } = useForm<PutSettingReq>({
    defaultValues: {
      userName: setting?.userName ?? undefined,
      userDescription: setting?.userDescription ?? undefined,
      instanceDescription: setting?.instanceDescription ?? undefined,
      maintainerName: setting?.maintainerName ?? undefined,
      maintainerEmail: setting?.maintainerEmail ?? undefined,
      themeColor: setting?.themeColor ?? undefined,
      objectStoreType: setting?.objectStoreType ?? undefined,
      objectStoreS3Bucket: setting?.objectStoreS3Bucket ?? undefined,
      objectStoreS3PublicUrlBase:
        setting?.objectStoreS3PublicUrlBase ?? undefined,
      objectStoreLocalFileSystemBasePath:
        setting?.objectStoreLocalFileSystemBasePath ?? undefined,
    },
  });
  const { mutate: putSetting, isLoading, error } = usePutSettingMutation();

  const onSubmit: SubmitHandler<PutSettingReq> = (data) => {
    putSetting(data);
    modalRef.current?.close();
  };

  return (
    <>
      <button
        className="flex items-center"
        onClick={() => {
          modalRef.current?.showModal();
        }}
      >
        <Cog6ToothIcon className="mr-3 inline h-10 w-10" />
        <span className="text-lg">Settings</span>
      </button>
      <dialog ref={modalRef} className="modal">
        <div className="modal-box">
          <h2 className="mb-4 text-lg font-bold">Settings</h2>
          <form className="form-control" onSubmit={handleSubmit(onSubmit)}>
            <label className="label label-text">User name</label>
            <input
              type="text"
              className="input input-bordered w-full"
              {...register("userName")}
            />
            <label className="label label-text">User description</label>
            <textarea
              className="textarea textarea-bordered w-full"
              {...register("userDescription")}
            />
            <label className="label label-text">Instance description</label>
            <textarea
              className="textarea textarea-bordered w-full"
              {...register("instanceDescription")}
            />
            <label className="label label-text">Maintainer name</label>
            <input
              type="text"
              className="input input-bordered w-full"
              {...register("maintainerName")}
            />
            <label className="label label-text">Maintainer email</label>
            <input
              type="email"
              className="input input-bordered w-full"
              {...register("maintainerEmail")}
            />
            <label className="label label-text">Object store type</label>
            <label className="label cursor-pointer">
              <span className="label-text">Local filesystem</span>
              <input
                type="radio"
                className="radio"
                value="localFileSystem"
                {...register("objectStoreType")}
              />
            </label>
            <label className="label cursor-pointer">
              <span className="label-text">S3</span>
              <input
                type="radio"
                className="radio"
                value="s3"
                {...register("objectStoreType")}
              />
            </label>
            <label className="label label-text">Object store S3 bucket</label>
            <input
              type="text"
              className="input input-bordered w-full"
              {...register("objectStoreS3Bucket")}
            />
            <label className="label label-text">
              Object store S3 public URL base
            </label>
            <input
              type="text"
              className="input input-bordered w-full"
              {...register("objectStoreS3PublicUrlBase")}
            />
            <label className="label label-text">
              Object store local filesystem base path
            </label>
            <input
              type="text"
              className="input input-bordered w-full"
              {...register("objectStoreLocalFileSystemBasePath")}
            />
            <input
              type="submit"
              className="btn btn-primary mt-4"
              value="Save"
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
