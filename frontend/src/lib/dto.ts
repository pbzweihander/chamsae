import * as z from "zod";

export const Id = z.string().length(26);

export const ErrorResponse = z.object({
  id: Id,
  error: z.string(),
});

export async function throwError(resp: Response): Promise<never> {
  const body = await resp.text();
  try {
    const parsed = ErrorResponse.parse(JSON.parse(body));
    throw new Error(
      `failed to login. status code: ${resp.status}, error id: ${parsed.id}, message: ${parsed.error}`,
    );
  } catch {
    throw new Error(`failed to login. status code: ${resp.status}, message: ${body}`);
  }
}

export const IdResponse = z.object({
  id: Id,
});

export const NameResponse = z.object({
  name: z.string(),
});

export const User = z.object({
  id: Id,
  handle: z.string(),
  name: z.nullable(z.string()),
  description: z.nullable(z.string()),
  host: z.string(),
  uri: z.string().url(),
  avatarUrl: z.nullable(z.string().url()),
  bannerUrl: z.nullable(z.string().url()),
  manuallyApprovesFollowers: z.boolean(),
  isBot: z.boolean(),
});

export const Visibility = z.enum(["public", "home", "followers", "directMessage"]);

export const Mention = z.object({
  userUri: z.string().url(),
  name: z.string(),
});

export const File = z.object({
  mediaType: z.string(),
  url: z.string().url(),
  alt: z.nullable(z.string()),
});

export const Emoji = z.object({
  name: z.string(),
  mediaType: z.string(),
  imageUrl: z.string().url(),
});

export const CreateContentReaction = z.object({
  content: z.string(),
});

export const CreateEmojiReaction = z.object({
  emojiName: z.string(),
});

export const CreateReaction = z.union([CreateContentReaction, CreateEmojiReaction]);

export const Reaction = z.object({
  id: Id,
  user: z.nullable(User),
  content: z.string(),
  emoji: z.nullable(Emoji),
});

export const Post = z.object({
  id: Id,
  createdAt: z.string().datetime({ offset: true }),
  replyId: z.nullable(Id),
  repliesId: z.array(Id),
  repostId: z.nullable(Id),
  text: z.string(),
  title: z.nullable(z.string()),
  user: z.nullable(User),
  visibility: Visibility,
  isSensitive: z.boolean(),
  uri: z.string().url(),
  files: z.array(File),
  reactions: z.array(Reaction),
  mentions: z.array(Mention),
  emojis: z.array(Emoji),
  hashtags: z.array(z.string()),
});

export const CreatePost = z.object({
  replyId: z.nullable(Id),
  repostId: z.nullable(Id),
  text: z.string(),
  title: z.nullable(z.string()),
  visibility: Visibility,
  isSensitive: z.boolean(),
  files: z.array(Id),
  mentions: z.array(Mention),
  emojis: z.array(z.string()),
  hashtags: z.array(z.string()),
});

export const LocalFile = z.object({
  id: Id,
  posted: z.boolean(),
  emojiName: z.nullable(z.string()),
  mediaType: z.string(),
  url: z.string().url(),
  alt: z.nullable(z.string()),
});

export const LocalEmoji = z.object({
  name: z.string(),
  createdAt: z.string().datetime({ offset: true }),
  mediaType: z.string(),
  imageUrl: z.string().url(),
});

export const CreateEmoji = z.object({
  fileId: Id,
  name: z.string(),
});

export const Follow = z.intersection(User, z.object({ accepted: z.boolean() }));

export const CreateFollow = z.object({
  toId: Id,
});

export const Setting = z.object({
  userName: z.nullable(z.string()),
  userDescription: z.nullable(z.string()),
  instanceDescription: z.nullable(z.string()),
  avatarFileId: z.nullable(Id),
  bannerFileId: z.nullable(Id),
  maintainerName: z.nullable(z.string()),
  maintainerEmail: z.nullable(z.string().email()),
  themeColor: z.nullable(z.string()),
});

export const Object = z.union([User, Post]);

export const Report = z.object({
  from: User,
  content: z.string(),
});

export const CreateReport = z.object({
  userId: Id,
  content: z.string(),
});
