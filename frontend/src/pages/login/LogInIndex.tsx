import { SubmitHandler, useForm } from "react-hook-form";
import z from "zod";

import { CreatePost } from "../../dto";
import { useNotes, usePostNoteMutation } from "../../queries/note";

export function LogInIndexPage() {
  const { data: notes } = useNotes();
  const { register, handleSubmit, reset } =
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
  };

  return (
    <div className="relative flex h-full w-full">
      <div className="h-full w-full overflow-y-scroll py-10">
        {(notes ?? []).map((note) => (
          <div
            key={note.id}
            className={`chat chat-${note.user != null ? "start" : "end"}`}
          >
            {note.user && (
              <div className="chat-header">
                {note.user.name != null ? (
                  <span>
                    {note.user.name}
                    <span className="ml-2 text-gray-500">
                      @{note.user.handle}
                    </span>
                  </span>
                ) : (
                  <span>@{note.user.handle}</span>
                )}
              </div>
            )}
            <div className="chat-bubble">{note.text}</div>
          </div>
        ))}
      </div>
      <form
        className="chat chat-end absolute bottom-4 right-4"
        onSubmit={handleSubmit(onSubmit)}
      >
        <input type="hidden" value="public" {...register("visibility")} />
        <div className="chat-bubble chat-bubble-primary">
          <textarea
            className="textarea w-full text-base-content"
            placeholder="Jot something..."
            required
            {...register("text")}
          />
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
    </div>
  );
}
