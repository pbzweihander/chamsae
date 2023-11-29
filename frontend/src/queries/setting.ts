import { useQueryClient } from "react-query";
import z from "zod";

import { useJsonQuery } from ".";
import { useJsonMutation, type JsonMutationRet } from ".";
import { Setting } from "../dto";

const SETTING_KEY = ["setting"];

export function useSetting() {
  return useJsonQuery(z.optional(Setting), SETTING_KEY, "/api/setting");
}

export interface InitializeSettingReq {
  instanceName: string;
  userHandle: string;
  userPassword: string;
}

export function useInitializeSettingMutation(): JsonMutationRet<
  InitializeSettingReq,
  z.ZodVoid
> {
  const queryClient = useQueryClient();
  return useJsonMutation("POST", "/api/setting/initial", z.void(), {
    onSuccess: () => {
      queryClient.invalidateQueries(SETTING_KEY);
    },
  });
}

export interface PutSettingReq {
  userName?: string;
  userDescription?: string;
  instanceDescription?: string;
  avatarFileId?: string;
  bannerFileId?: string;
  maintainerName?: string;
  maintainerEmail?: string;
  themeColor?: string;
}

export function usePutSettingMutation(): JsonMutationRet<
  PutSettingReq,
  typeof Setting
> {
  const queryClient = useQueryClient();
  return useJsonMutation("PUT", "/api/setting", Setting, {
    onSuccess: () => {
      queryClient.invalidateQueries(SETTING_KEY);
    },
  });
}
