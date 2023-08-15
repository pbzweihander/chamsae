import Link from "next/link";

export const metadata = {
  title: "Feed",
};

export default function Feed() {
  return (
    <div className="flex flex-col items-stretch">
      <FeedItem id="foo">Foo</FeedItem>
      <FeedItem id="bar">Bar</FeedItem>
      <FeedItem id="baz">Baz</FeedItem>
    </div>
  );
}

interface FeedItemProps {
  id: string;
  children?: React.ReactNode;
}

function FeedItem(props: FeedItemProps) {
  return (
    <Link
      href={`/post/${props.id}`}
      className="border border-b-0 last:border-b first:rounded-t last:rounded-b"
    >
      <div className="p-4">
        {props.children}
      </div>
    </Link>
  );
}
