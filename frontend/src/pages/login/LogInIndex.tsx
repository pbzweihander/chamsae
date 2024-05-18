import { PlusIcon } from "@heroicons/react/24/outline";
import { Fragment, useEffect } from "react";

import BottomNewChirp from "../../components/NewChirp/BottomNewChirp";
import { useNotes } from "../../queries/note";

export function LogInIndexPage() {
  const { data, fetchNextPage, hasNextPage, isFetchingNextPage } = useNotes();

  useEffect(() => {
    console.log(data);
  }, [data]);

  return (
    <div className="relative flex h-full w-full">
      <div className="h-full w-full overflow-y-scroll p-6">
        {(data?.pages ?? []).map((page, i) => (
          <Fragment key={i}>
            {page.map((note) => (
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
          </Fragment>
        ))}
        <div className="mt-4 flex w-full justify-center">
          {isFetchingNextPage ? (
            <span className="loading loading-spinner loading-lg" />
          ) : (
            hasNextPage && (
              <button
                onClick={() => {
                  fetchNextPage();
                }}
              >
                <PlusIcon className="h-10 w-10" />
              </button>
            )
          )}
        </div>
      </div>
      <BottomNewChirp />
    </div>
  );
}
