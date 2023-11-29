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
    },
  });
  const { mutate: putSetting, isLoading, error } = usePutSettingMutation();

  const onSubmit: SubmitHandler<PutSettingReq> = (data) => {
    putSetting(data);
  };

  return (
    <>
      <button
        onClick={() => {
          modalRef.current?.showModal();
        }}
      >
        <Cog6ToothIcon className="h-10 w-10" />
      </button>
      <dialog ref={modalRef} className="modal">
        <form
          className="form-control modal-box"
          onSubmit={handleSubmit(onSubmit)}
        >
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
          <input
            type="submit"
            className="btn btn-primary mt-4"
            value="Save"
            disabled={isLoading}
          />
          {error && <div className="mt-5 text-error">{error.message}</div>}
        </form>
        <form method="dialog" className="modal-backdrop">
          <button>close</button>
        </form>
      </dialog>
    </>
  );
}
