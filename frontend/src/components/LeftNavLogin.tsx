import { useRef } from "react";
import { SubmitHandler, useForm } from "react-hook-form";

import { LoginReq, useLoginMutation } from "../queries/auth";

export default function LeftNavLogin() {
  const modalRef = useRef<HTMLDialogElement>(null);
  const { register, handleSubmit } = useForm<LoginReq>();
  const {
    mutate: login,
    isLoading: isLoginLoading,
    error,
  } = useLoginMutation(() => {
    modalRef.current?.close();
  });

  const onSubmit: SubmitHandler<LoginReq> = (data) => {
    login(data);
  };

  return (
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
  );
}
