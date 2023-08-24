import { parse as parseEmoji } from "twemoji-parser";

import type { CustomEmojiMapper } from "./Mfm";

type Props =
  | { custom?: false; code: string }
  | { custom: true; code: string; srcMapper?: CustomEmojiMapper };

export default function MfmEmoji(props: Props) {
  let containerClassName = "inline-block align-bottom overflow-hidden h-6";
  let id;
  let src;
  let alt;
  if (props.custom) {
    id = props.code;
    src = props.srcMapper?.(props.code);
    alt = `:${props.code}:`;
  } else {
    containerClassName += " aspect-square";
    const parsed = parseEmoji(props.code, {
      buildUrl(codepoints) {
        return `https://cdn.jsdelivr.net/gh/twitter/twemoji@14.0.2/assets/svg/${codepoints}.svg`;
      },
      assetType: "svg",
    });
    if (parsed[0]) {
      id = parsed[0].text;
      src = parsed[0].url;
    } else {
      id = props.code;
      src = undefined;
    }
    alt = props.code;
  }

  return (
    <div key={id} className={containerClassName}>
      <img src={src} alt={alt} loading="lazy" decoding="async" className="h-full" />
    </div>
  );
}
