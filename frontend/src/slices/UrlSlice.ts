import { createSlice } from "@reduxjs/toolkit";
import type { PayloadAction } from "@reduxjs/toolkit";

interface UrlState {
  url: string[];
  key: number;
}

const initialState: UrlState = {
  url: [],
  key: 0,
};

export const urlSlice = createSlice({
  name: "url",
  initialState,
  reducers: {
    storeUrl: (state, action: PayloadAction<string>) => {
      state.url.push(action.payload);
      state.key += 1;
    },
    resetUrl: () => {
      return initialState;
    },
  },
});

export const { storeUrl, resetUrl } = urlSlice.actions;

export default urlSlice.reducer;
