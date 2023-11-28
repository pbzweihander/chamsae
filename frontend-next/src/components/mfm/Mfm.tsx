import { parse as parseMfm } from "mfm-js";

import MfmRenderer from "./MfmRenderer";

export type CustomEmojiMapper = (code: string) => string | undefined;

interface Props {
  content: string;
  customEmojiMapper?: CustomEmojiMapper;
}

export default function Mfm(props: Props) {
  const parsed = parseMfm(props.content);
  return <MfmRenderer nodes={parsed} customEmojiMapper={props.customEmojiMapper} />;
}
