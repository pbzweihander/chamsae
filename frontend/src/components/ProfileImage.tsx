import clsx from "clsx";
import Image, { type ImageProps } from "next/image";

interface ProfileImageProps extends Omit<Partial<ImageProps>, "width" | "height" | "fill"> {
  size?: "small" | "medium" | "large";
}

export default function ProfileImage(props: ProfileImageProps) {
  const { size = "medium", src, alt = "", className, ...imageProps } = props;
  const computedClassName = clsx(
    "relative overflow-hidden rounded-full bg-gray-100 aspect-square",
    {
      "w-12": size === "small",
      "w-16": size === "medium",
      "w-24": size === "large",
    },
    className,
  );

  return (
    <div className={computedClassName}>
      {src != null && <Image src={src} alt={alt} {...imageProps} fill />}
    </div>
  );
}
