import { MfmFn } from "mfm-js";
import type { CSSProperties } from "react";

import { CustomEmojiMapper } from "./Mfm";
import MfmRenderer from "./MfmRenderer";

function parseFloatWithDefault(value: string | true | undefined, defaultValue: string): string {
  if (typeof value !== "string" || !/^\d+(:?\.\d+)?$/.test(value)) {
    return defaultValue;
  }
  return value;
}

export default function MfmFunction(
  { props, children, customEmojiMapper }: MfmFn & { customEmojiMapper?: CustomEmojiMapper },
) {
  let className;
  let style: CSSProperties | undefined;
  switch (props.name) {
    case "x2":
      className = "text-[2em]";
      break;
    case "x3":
      className = "text-[3em]";
      break;
    case "x4":
      className = "text-[4em]";
      break;
    case "flip":
      className = "inline-block -scale-x-100";
      break;
    case "scale": {
      const x = parseFloatWithDefault(props.args.x, "1");
      const y = parseFloatWithDefault(props.args.y, "1");
      className = "inline-block";
      style = { transform: `scale(${x}, ${y})` };
      break;
    }
    case "position": {
      const x = parseFloatWithDefault(props.args.x, "0");
      const y = parseFloatWithDefault(props.args.y, "0");
      className = "inline-block";
      style = { transform: `translate(${x}em, ${y}em)` };
      break;
    }
    case "rotate": {
      const degrees = parseFloatWithDefault(props.args.deg, "90");
      className = "inline-block";
      style = { transform: `rotate(${degrees}deg)` };
      break;
    }
  }

  if (!className && !style) {
    return <MfmRenderer nodes={children} customEmojiMapper={customEmojiMapper} />;
  }

  return (
    <span className={className} style={style}>
      <MfmRenderer nodes={children} customEmojiMapper={customEmojiMapper} />
    </span>
  );
}
