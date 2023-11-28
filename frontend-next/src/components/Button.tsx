import clsx from "clsx";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  kind?: "primary" | "secondary" | "warning";
  label: string;
  children?: undefined;
}

export default function Button(props: ButtonProps) {
  const { kind = "secondary", label, className, ...buttonProps } = props;

  const computedClassName = clsx(
    "border rounded px-4 py-2 transition",
    {
      "border-transparent bg-emerald-600 text-white hover:bg-emerald-500 active:bg-emerald-600":
        kind === "primary",
      "border-black bg-white hover:bg-gray-100 active:bg-gray-200": kind === "secondary",
      "border-red-500 bg-white text-red-600 hover:bg-red-100 active:bg-red-200": kind === "warning",
    },
    className,
  );

  return <button className={computedClassName} {...buttonProps}>{label}</button>;
}
