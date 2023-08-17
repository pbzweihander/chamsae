import { apiUrl } from "@/lib/api";
import getAccessKeyOrRedirect from "@/lib/api/getAccessKeyOrRedirect";
import { Post, throwError } from "@/lib/dto";
import Link from "next/link";
import * as z from "zod";

export const metadata = {
  title: "Feed",
};

async function getPosts(): Promise<z.infer<typeof Post>[]> {
  const accessKey = await getAccessKeyOrRedirect();

  const resp = await fetch(apiUrl("/api/post"), {
    headers: {
      "authorization": `Bearer ${accessKey}`,
    },
  });
  if (!resp.ok) {
    await throwError(resp);
  }
  return z.array(Post).parse(await resp.json());
}

export default async function Feed() {
  const posts = await getPosts();
  return (
    <div className="flex flex-col items-stretch">
      {posts.map(post => <FeedItem id={post.id} key={post.id}>{post.text}</FeedItem>)}
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
