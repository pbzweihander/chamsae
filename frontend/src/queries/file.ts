import { useContext } from "react";
import { useMutation, useQueryClient } from "react-query";
import * as z from "zod";

import { MutationRet, useInfiniteJsonQuery } from ".";
import { AccessKeyContext } from "../contexts/auth";
import { throwError, LocalFile, Id } from "../dto";

const FILES_KEY = ["notes"];

export function useLocalFiles() {
  const params = new URLSearchParams();
  params.set("size", "9");
  return useInfiniteJsonQuery(
    z.array(LocalFile),
    FILES_KEY,
    "/api/file",
    params,
  );
}

export interface PostLocalFileQuery {
  file: File;
  mediaType: string;
  alt?: string;
}

export function useLocalFileUploadMutation(
  onSuccess: () => void,
): MutationRet<PostLocalFileQuery> {
  const queryClient = useQueryClient();
  const [accessKey] = useContext(AccessKeyContext);

  return useMutation(
    async (payload) => {
      const headers: HeadersInit = {
        "content-type": "application/octet-stream",
      };
      if (accessKey.length > 0) {
        headers["authorization"] = `Bearer ${accessKey}`;
      }
      const resp = await fetch(
        `/api/file?mediaType=${payload.mediaType}${
          payload.alt ? "&alt=" + payload.alt : ""
        }`,
        {
          method: "POST",
          headers,
          body: payload.file,
        },
      );
      if (!resp.ok) {
        await throwError(resp);
      }
    },
    {
      onSuccess: () => {
        queryClient.invalidateQueries(FILES_KEY);
        onSuccess();
      },
    },
  );
}

export function useLocalFileDeleteMutation(
  onSuccess: () => void,
): MutationRet<z.infer<typeof Id>> {
  const queryClient = useQueryClient();
  const [accessKey] = useContext(AccessKeyContext);

  return useMutation(
    async (id) => {
      const headers: HeadersInit = {};
      if (accessKey.length > 0) {
        headers["authorization"] = `Bearer ${accessKey}`;
      }
      const resp = await fetch(`/api/file/${id}`, {
        method: "DELETE",
        headers,
      });
      if (!resp.ok) {
        await throwError(resp);
      }
    },
    {
      onSuccess: () => {
        queryClient.invalidateQueries(FILES_KEY);
        onSuccess();
      },
    },
  );
}
