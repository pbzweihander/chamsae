import { QueryClient, QueryClientProvider } from "react-query";
import { BrowserRouter, Route, Routes } from "react-router-dom";

import Layout from "./components/Layout";
import { AccessKeyContextProvider } from "./contexts/auth";
import IndexPage from "./pages/Index";
import NotFoundPage from "./pages/NotFound";
import NotePage from "./pages/Note";
import PersonPage from "./pages/Person";

const queryClient = new QueryClient();

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AccessKeyContextProvider>
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<Layout />}>
              <Route errorElement={<NotFoundPage />}>
                <Route index element={<IndexPage />} />
                <Route path="note/:id" element={<NotePage />} />
                <Route path="person/" element={<PersonPage />} />
              </Route>
            </Route>
          </Routes>
        </BrowserRouter>
      </AccessKeyContextProvider>
    </QueryClientProvider>
  );
}
