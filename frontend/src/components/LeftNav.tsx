import { ArrowLeftOnRectangleIcon } from "@heroicons/react/24/outline";
import { useRef } from "react";
import { SubmitHandler, useForm } from "react-hook-form";

import {
  LoginReq,
  useIsAuthed,
  useLoginMutation,
  useLogoutMutation,
} from "../queries/auth";
import { useSetting } from "../queries/setting";

export default function LeftNav() {
  const modalRef = useRef<HTMLDialogElement>(null);
  const { register, handleSubmit } = useForm<LoginReq>();
  const isAuthed = useIsAuthed() ?? false;
  const { data: setting } = useSetting();
  const {
    mutate: login,
    isLoading: isLoginLoading,
    error,
  } = useLoginMutation(() => {
    modalRef.current?.close();
  });
  const { mutate: logout, isLoading: isLogoutLoading } = useLogoutMutation();

  const onSubmit: SubmitHandler<LoginReq> = (data) => {
    login(data);
  };

  return (
    <div className="flex h-full w-64 flex-col p-4">
      <div className="flex-grow" />
      {isAuthed ? (
        setting && (
          <div className="flex w-full items-center p-2">
            {setting.avatarFileId && (
              <img
                className="avatar w-10 rounded-full"
                src={`/file/${setting.avatarFileId}`}
              />
            )}
            {setting.userName != null ? (
              <span>
                {setting.userName}
                <span className="text-neutral-content ml-2">
                  @{setting.userHandle}
                </span>
              </span>
            ) : (
              <span>@{setting.userHandle}</span>
            )}
            <button
              onClick={() => {
                logout();
              }}
              disabled={isLogoutLoading}
            >
              <ArrowLeftOnRectangleIcon className="text-error ml-4 h-6 w-6" />
            </button>
          </div>
        )
      ) : (
        <>
          <button
            className="btn btn-neutral w-full"
            onClick={() => {
              modalRef.current?.showModal();
            }}
          >
            Login
          </button>
          <dialog ref={modalRef} className="modal">
            <form
              className="modal-box form-control"
              onSubmit={handleSubmit(onSubmit)}
            >
              <label className="label label-text">Password</label>
              <input
                type="password"
                className="input input-bordered w-full"
                required
                {...register("password")}
              />
              <input
                type="submit"
                className="btn btn-primary mt-4"
                disabled={isLoginLoading}
              />
              {error && <div className="text-error mt-5">{error.message}</div>}
            </form>
            <form method="dialog" className="modal-backdrop">
              <button>close</button>
            </form>
          </dialog>
        </>
      )}
    </div>
  );
}
