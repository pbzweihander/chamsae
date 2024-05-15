import { useContext } from "react";
import {
  UseQueryResult,
  useQuery,
  UseMutationOptions,
  UseMutationResult,
  useMutation,
  useInfiniteQuery,
  UseInfiniteQueryResult,
  UseQueryOptions,
} from "react-query";
import z from "zod";

import { AccessKeyContext } from "../contexts/auth";
import { throwError, Id } from "../dto";

export type UseJsonQueryResult<T extends z.ZodTypeAny> = UseQueryResult<
  z.infer<T>,
  Error
>;

export function useJsonQuery<T extends z.ZodTypeAny>(
  schema: T,
  key: string[],
  url: string,
  params?: URLSearchParams,
  options?: UseQueryOptions<T, Error>,
): UseJsonQueryResult<T> {
  const [accessKey] = useContext(AccessKeyContext);

  if (params) {
    url = `${url}?${params.toString()}`;
  }

  return useQuery({
    queryKey: [...key, accessKey, params?.toString()],
    queryFn: async () => {
      const resp = await fetch(url, {
        headers:
          accessKey.length > 0
            ? {
                authorization: `Bearer ${accessKey}`,
              }
            : undefined,
      });
      if (schema.isOptional() && resp.status == 404) {
        return undefined;
      }
      if (!resp.ok) {
        await throwError(resp);
      }
      if (schema instanceof z.ZodVoid) {
        return;
      }
      return schema.parse(await resp.json());
    },
    ...options,
  });
}

export type UseInfiniteJsonQueryResult<T extends z.ZodTypeAny> =
  UseInfiniteQueryResult<z.infer<T>, Error>;

export function useInfiniteJsonQuery<
  T extends z.ZodType<{ id: z.infer<typeof Id> }[]>,
>(
  schema: T,
  key: string[],
  url: string,
  params?: URLSearchParams,
): UseInfiniteJsonQueryResult<T> {
  const [accessKey] = useContext(AccessKeyContext);

  const strictParams = params ?? new URLSearchParams();

  return useInfiniteQuery(
    [...key, accessKey, strictParams.toString()],
    async ({ pageParam }) => {
      if (pageParam) {
        strictParams.set("after", pageParam);
      }
      const finalUrl = `${url}?${strictParams.toString()}`;

      const resp = await fetch(finalUrl, {
        headers:
          accessKey.length > 0
            ? {
                authorization: `Bearer ${accessKey}`,
              }
            : undefined,
      });
      if (schema.isOptional() && resp.status == 404) {
        return undefined;
      }
      if (!resp.ok) {
        await throwError(resp);
      }
      if (schema instanceof z.ZodVoid) {
        return;
      }
      return schema.parse(await resp.json());
    },
    {
      getNextPageParam: (last) => {
        if (last && last.length > 0) {
          return last[last.length - 1].id;
        }
      },
    },
  );
}

export type MutationRet<Req, Resp = void, Err = TypeError> = UseMutationResult<
  Resp,
  Err,
  Req,
  undefined
>;
export type MutationOption<Req, Resp = void, Err = TypeError> = Omit<
  UseMutationOptions<Resp, Err, Req, undefined>,
  "mutationFn"
>;

export type JsonMutationRet<Req, Resp extends z.ZodTypeAny> = MutationRet<
  Req,
  z.infer<Resp>,
  Error
>;
export type JsonMutationOption<Req, Resp extends z.ZodTypeAny> = MutationOption<
  Req,
  z.infer<Resp>,
  Error
>;

export function useJsonMutation<Req, Resp extends z.ZodTypeAny>(
  method: "POST" | "PUT" | "DELETE",
  url: string,
  schema: Resp,
  options?: JsonMutationOption<Req, Resp>,
): JsonMutationRet<Req, Resp> {
  const [accessKey] = useContext(AccessKeyContext);

  return useMutation(async (payload) => {
    const headers: HeadersInit = {
      "content-type": "application/json",
    };
    if (accessKey.length > 0) {
      headers["authorization"] = `Bearer ${accessKey}`;
    }
    const resp = await fetch(url, {
      method,
      headers,
      body: JSON.stringify(payload),
    });
    if (schema.isOptional() && resp.status == 404) {
      return undefined;
    }
    if (!resp.ok) {
      await throwError(resp);
    }
    if (schema instanceof z.ZodVoid) {
      return;
    }
    return schema.parse(await resp.json());
  }, options);
}
