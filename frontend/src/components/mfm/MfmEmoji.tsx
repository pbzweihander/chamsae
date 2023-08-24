import twemoji from "twemoji";

import type { CustomEmojiMapper } from "./Mfm";

// From twemoji source code
function rawUnicodeEmojiToId(emoji: string): string {
  return twemoji.convert.toCodePoint(
    emoji.includes("\u200d") ? emoji : emoji.replaceAll("\ufe0f", ""),
  );
}

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
    id = rawUnicodeEmojiToId(props.code);
    src = `https://cdn.jsdelivr.net/gh/twitter/twemoji@14.0.2/assets/svg/${id}.svg`;
    alt = props.code;
  }

  return (
    <div key={id} className={containerClassName}>
      <img src={src} alt={alt} loading="lazy" decoding="async" className="h-full" />
    </div>
  );
}
