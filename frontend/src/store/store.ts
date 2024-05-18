import { configureStore } from "@reduxjs/toolkit";

import UrlSlice from "../slices/UrlSlice";

export const store = configureStore({
  reducer: { UrlSlice },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
