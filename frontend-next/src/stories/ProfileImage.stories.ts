import type { Meta, StoryObj } from "@storybook/react";

import ProfileImage from "@/components/ProfileImage";
import profileImageData from "./assets/plachta.png";

const meta = {
  title: "Basic Components/ProfileImage",
  component: ProfileImage,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: {
    src: profileImageData,
    alt: "Profile: tirr-c",
  },
} satisfies Meta<typeof ProfileImage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Small: Story = {
  args: {
    size: "small",
  },
};

export const Medium: Story = {
  args: {
    size: "medium",
  },
};

export const Large: Story = {
  args: {
    size: "large",
  },
};

export const None: Story = {
  args: {
    src: undefined,
    alt: undefined,
  },
};
