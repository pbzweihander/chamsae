import { TrashIcon } from "@heroicons/react/24/outline";
import { useEffect } from "react";
import { SubmitHandler, useForm } from "react-hook-form";
import z from "zod";

import { CreatePost } from "../../dto";
import { usePostNoteMutation } from "../../queries/note";
import { resetUrl } from "../../slices/UrlSlice";
import { useAppDispatch, useAppSelector } from "../../store/hooks";
import BottomUpload from "./BottomUpload";

export default function BottomNewChirp() {
  const dispatch = useAppDispatch();
  const pictureUrl = useAppSelector((state) => state.UrlSlice);

  const { register, handleSubmit, setValue, reset } =
    useForm<z.infer<typeof CreatePost>>();
  const {
    mutate: postNote,
    isLoading,
    error,
  } = usePostNoteMutation(() => {
    reset();
  });

  const onSubmit: SubmitHandler<z.infer<typeof CreatePost>> = (data) => {
    postNote(data);
    deletePicture();
  };

  const deletePicture = () => {
    dispatch(resetUrl());
  };

  useEffect(() => {
    const ulid: string[] = pictureUrl.url.map((el: string) =>
      el.substring(el.length - 26, el.length),
    );
    setValue("files", ulid);
  }, [setValue, pictureUrl.url]);

  return (
    <>
      <form
        className="chat chat-end absolute bottom-4 right-4"
        onSubmit={handleSubmit(onSubmit)}
      >
        <input type="hidden" value="public" {...register("visibility")} />
        <div className="chat-bubble chat-bubble-primary">
          {/*           <textarea
      className="textarea w-full resize-y bg-transparent text-base-content"
      placeholder="Jot something..."
      required
      {...register("text")}
    /> */}
          <div className="flex items-center">
            <BottomUpload />
            <div className="">
              {pictureUrl.key > 0 && (
                <div>
                  {pictureUrl.url.map((value, i) => (
                    <div key={i}>
                      <input
                        type="hidden"
                        value={pictureUrl.url}
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
    </>
  );
}
