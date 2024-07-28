import { SubmitHandler, useForm } from "react-hook-form";

import {
  InitializeSettingReq,
  useInitializeSettingMutation,
} from "../../queries/setting";

export default function InitializePage() {
  const { register, handleSubmit } = useForm<InitializeSettingReq>();
  const {
    mutate: initialize,
    isLoading,
    error,
  } = useInitializeSettingMutation();

  const onSubmit: SubmitHandler<InitializeSettingReq> = (data) => {
    initialize(data);
  };

  return (
    <dialog className="modal modal-open">
      <div className="modal-box">
        <form className="form-control" onSubmit={handleSubmit(onSubmit)}>
          <h2 className="text-xl font-bold">Initialize instance</h2>
          <label className="label label-text">Instance name</label>
          <input
            type="text"
            className="input input-bordered w-full"
            required
            {...register("instanceName")}
          />
          <label className="label label-text">User handle</label>
          <input
            type="text"
            className="input input-bordered w-full"
            autoComplete="username"
            placeholder="admin"
            required
            {...register("userHandle")}
          />
          <label className="label label-text">User password</label>
          <input
            type="password"
            className="input input-bordered w-full"
            required
            autoComplete="new-password"
            {...register("userPassword")}
          />
          <label className="label label-text">Object store local file system base path</label>
          <input
            type="text"
            className="input input-bordered w-full"
            placeholder="./files/"
            required
            {...register("objectStoreLocalFileSystemBasePath")}
          />
          <input
            type="submit"
            className="btn btn-primary mt-4"
            value="Initialize"
            disabled={isLoading}
          />
          {error && <div className="mt-5 text-error">{error.message}</div>}
        </form>
      </div>
    </dialog>
  );
}
