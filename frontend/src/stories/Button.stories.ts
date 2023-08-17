import type { Meta, StoryObj } from "@storybook/react";

import Button from "@/components/Button";

const meta = {
  title: "Basic Components/Button",
  component: Button,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    kind: "primary",
    label: "Button",
  },
};

export const Secondary: Story = {
  args: {
    kind: "secondary",
    label: "Button",
  },
};

export const Warning: Story = {
  args: {
    kind: "warning",
    label: "Button",
  },
};
