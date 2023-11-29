import { useIsAuthed } from "../queries/auth";
import { useSetting } from "../queries/setting";
import ErrorPage from "./Error";
import LoadingPage from "./Loading";
import { LogInIndexPage } from "./login/LogInIndex";
import InitializePage from "./logout/Initialize";
import LogOutIndexPage from "./logout/LogOutIndex";

export default function IndexPage() {
  const { data: setting, isLoading, error } = useSetting();
  const isAuthed = useIsAuthed();

  if (isLoading || isAuthed == null) {
    return <LoadingPage />;
  }
  if (error != null) {
    return <ErrorPage error={error} />;
  }
  if (setting == null) {
    return <InitializePage />;
  }
  if (isAuthed) {
    return <LogInIndexPage />;
  } else {
    return <LogOutIndexPage />;
  }
}
