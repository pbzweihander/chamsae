import type * as Mfm from "mfm-js";
import Link from "next/link";

import type { CustomEmojiMapper } from "./Mfm";
import MfmEmoji from "./MfmEmoji";
import MfmFunction from "./MfmFunction";

interface Props {
  nodes: Mfm.MfmNode[];
  customEmojiMapper?: CustomEmojiMapper;
}

export default function MfmRenderer({ nodes, customEmojiMapper }: Props) {
  const rendered = nodes.map((node, idx) => {
    switch (node.type) {
      case "text":
        return <MfmText key={idx} {...node.props} />;
      case "bold":
        return (
          <span key={idx} className="font-bold">
            <MfmRenderer nodes={node.children} customEmojiMapper={customEmojiMapper} />
          </span>
        );
      case "italic":
        return (
          <span key={idx} className="italic">
            <MfmRenderer nodes={node.children} customEmojiMapper={customEmojiMapper} />
          </span>
        );
      case "strike":
        return (
          <span key={idx} className="line-through">
            <MfmRenderer nodes={node.children} customEmojiMapper={customEmojiMapper} />
          </span>
        );
      case "center":
        return (
          <div key={idx} className="text-center">
            <MfmRenderer nodes={node.children} customEmojiMapper={customEmojiMapper} />
          </div>
        );
      case "small":
        return (
          <span key={idx} className="text-sm text-slate-400">
            <MfmRenderer nodes={node.children} customEmojiMapper={customEmojiMapper} />
          </span>
        );
      case "inlineCode":
        return (
          <code key={idx} className="font-mono bg-slate-100 rounded p-1">
            {node.props.code}
          </code>
        );
      case "mention":
        return <MfmMention key={idx} {...node.props} />;
      case "quote":
        return (
          <blockquote key={idx} className="border-l-4 border-slate-200 pl-2 text-slate-600">
            <MfmRenderer nodes={node.children} customEmojiMapper={customEmojiMapper} />
          </blockquote>
        );
      case "blockCode":
        return <MfmCodeBlock key={idx} {...node.props} />;
      case "hashtag":
        return <MfmHashtag key={idx} {...node.props} />;
      case "url":
      case "link":
        return <MfmUrlLike key={idx} {...node} customEmojiMapper={customEmojiMapper} />;
      case "plain":
        return (
          <MfmRenderer key={idx} nodes={node.children} customEmojiMapper={customEmojiMapper} />
        );
      case "unicodeEmoji":
        return <MfmEmoji key={idx} code={node.props.emoji} />;
      case "emojiCode":
        return <MfmEmoji key={idx} custom code={node.props.name} srcMapper={customEmojiMapper} />;
      case "fn":
        return <MfmFunction key={idx} {...node} customEmojiMapper={customEmojiMapper} />;
      default:
        return null;
    }
  });

  return <>{rendered}</>;
}

function MfmText({ text }: Mfm.MfmText["props"]) {
  const rendered: React.ReactNode[] = [];
  text.split(/\r?\n/g).forEach((line, idx) => {
    if (idx > 0) {
      rendered.push(<br key={idx} />);
    }
    rendered.push(line);
  });

  return <>{rendered}</>;
}

function MfmMention({ username, host }: Mfm.MfmMention["props"]) {
  return (
    <span className="text-emerald-700">
      {`@${username}`}
      {host && `@${host}`}
    </span>
  );
}

function MfmHashtag({ hashtag }: Mfm.MfmHashtag["props"]) {
  return (
    <span className="text-emerald-700">
      {`#${hashtag}`}
    </span>
  );
}

function MfmUrlLike(props: (Mfm.MfmUrl | Mfm.MfmLink) & { customEmojiMapper?: CustomEmojiMapper }) {
  let text: React.ReactNode;
  let href: string;
  if (props.type === "url") {
    text = props.props.url;
    href = props.props.url;
  } else {
    text = <MfmRenderer nodes={props.children} customEmojiMapper={props.customEmojiMapper} />;
    href = props.props.url;
  }

  return (
    <Link href={href} className="text-emerald-700">
      {text}
    </Link>
  );
}

function MfmCodeBlock({ code }: Mfm.MfmCodeBlock["props"]) {
  return (
    <pre className="font-mono bg-slate-100 rounded p-2">
      {code}
    </pre>
  );
}
