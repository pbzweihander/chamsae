import { MfmFn, MfmNode } from "mfm-js";

import { CustomEmojiMapper } from "./Mfm";
import MfmRenderer from "./MfmRenderer";

export default function MfmFunction(
  { props, children, customEmojiMapper }: MfmFn & { customEmojiMapper?: CustomEmojiMapper },
) {
  switch (props.name) {
    case "x2":
      return <MfmFnZoom scale={2} nodes={children} customEmojiMapper={customEmojiMapper} />;
    case "x3":
      return <MfmFnZoom scale={3} nodes={children} customEmojiMapper={customEmojiMapper} />;
    case "x4":
      return <MfmFnZoom scale={4} nodes={children} customEmojiMapper={customEmojiMapper} />;
    default:
      return <MfmRenderer nodes={children} customEmojiMapper={customEmojiMapper} />;
  }
}

function MfmFnZoom(
  { scale, nodes, customEmojiMapper }: {
    scale: number;
    nodes: MfmNode[];
    customEmojiMapper?: CustomEmojiMapper;
  },
) {
  const classNameMap = ["", "", "text-[2em]", "text-[3em]", "text-[4em]"];
  return (
    <span className={classNameMap[scale]}>
      <MfmRenderer nodes={nodes} customEmojiMapper={customEmojiMapper} />
    </span>
  );
}
