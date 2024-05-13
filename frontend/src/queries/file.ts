import { useContext } from "react";
import { useMutation } from "react-query";

import { MutationRet } from ".";
import { AccessKeyContext } from "../contexts/auth";
import { throwError } from "../dto";

export interface PostFileQuery {
  file: File;
  mediaType: string;
  alt?: string;
}

export function useFileUploadMutation(
  onSuccess: () => void,
): MutationRet<PostFileQuery> {
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
        onSuccess();
      },
    },
  );
}
