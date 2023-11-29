import { ReactNode, createContext, useEffect, useState } from "react";

export const AccessKeyContext = createContext<
  [string, (accessKey: string) => void]
>(["", () => {}]);

export function AccessKeyContextProvider({
  children,
}: {
  children: ReactNode;
}) {
  const [accessKey, setAccessKey] = useState(() => {
    return localStorage.getItem("accessKey") ?? "";
  });

  useEffect(() => {
    localStorage.setItem("accessKey", accessKey);
  }, [accessKey]);

  return (
    <AccessKeyContext.Provider value={[accessKey, setAccessKey]}>
      {children}
    </AccessKeyContext.Provider>
  );
}
