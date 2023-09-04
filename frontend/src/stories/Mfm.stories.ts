import type { Meta, StoryObj } from "@storybook/react";
import React from "react";

import Mfm from "@/components/mfm/Mfm";

import foodTestdata from "./assets/mfm-food.json";
import transformsTestdata from "./assets/mfm-transforms.json";
import plachta from "./assets/plachta.png";
import recruitSimulatorTestdata from "./assets/recruit.json";
import sendMoney from "./assets/send-money.png";
import unicodeEmojisTestdata from "./assets/spice.json";

const meta = {
  title: "Mfm/Mfm",
  component: Mfm,
  parameters: {
    layout: "centered",
  },
  decorators: [
    Story =>
      React.createElement(
        "div",
        { className: "border overflow-hidden w-[320px] p-2" },
        React.createElement(Story),
      ),
  ],
  args: {
    content: "",
    customEmojiMapper: code => {
      switch (code) {
        case "plachta":
          return plachta.src;
        case "send_money":
          return sendMoney.src;
        default:
          return undefined;
      }
    },
  },
  argTypes: {
    customEmojiMapper: {
      type: "function",
    },
  },
} satisfies Meta<typeof Mfm>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Simple: Story = {
  args: {
    content: "@tirr @pbzweihander@yuri.garden\n와! 샌즈! #언더테일",
  },
};

export const UnicodeEmojis: Story = {
  args: unicodeEmojisTestdata,
};

export const CustomEmojis: Story = {
  args: {
    content: ":plachta: :send_money:",
  },
};

export const EmojiZoom: Story = {
  args: {
    content: ":send_money:\n$[x2 :send_money:]\n$[x3 :send_money:]\n$[x4 :send_money:]",
  },
};

export const Transforms: Story = {
  args: transformsTestdata,
};

export const RecruitSimulator: Story = {
  args: recruitSimulatorTestdata,
};

export const Food: Story = {
  args: foodTestdata,
};
