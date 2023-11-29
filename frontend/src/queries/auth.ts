import { useContext } from "react";
import { useQuery } from "react-query";
import z from "zod";

import { JsonMutationRet, useJsonMutation } from ".";
import { AccessKeyContext } from "../contexts/auth";
import { Id } from "../dto";

export function useIsAuthed(): boolean | undefined {
  const [accessKey] = useContext(AccessKeyContext);

  const {
    data: resp,
    isLoading,
    isError,
  } = useQuery(["auth/check", accessKey], async () => {
    return await fetch("/api/auth/check", {
      headers: {
        authorization: `Bearer ${accessKey}`,
      },
    });
  });

  if (isError) {
    return false;
  }
  if (isLoading || resp == null) {
    return undefined;
  }
  return resp.ok;
}

export interface LoginReq {
  password: string;
}

const LoginResp = z.object({
  token: Id,
});

export function useLoginMutation(
  onSuccess: () => void,
): JsonMutationRet<LoginReq, typeof LoginResp> {
  const [, setAccessKey] = useContext(AccessKeyContext);
  return useJsonMutation("POST", "/api/auth/login", LoginResp, {
    onSuccess: (resp) => {
      setAccessKey(resp.token);
      onSuccess();
    },
  });
}

export function useLogoutMutation(): JsonMutationRet<void, z.ZodVoid> {
  const [, setAccessKey] = useContext(AccessKeyContext);
  return useJsonMutation("POST", "/api/auth/logout", z.void(), {
    onSuccess: () => {
      setAccessKey("");
    },
  });
}
