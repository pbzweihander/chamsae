import * as z from "zod";

export const Id = z.string().length(26);

export const IdResponse = z.object({
  id: Id,
});

export const NameResponse = z.object({
  name: z.string(),
});

export const User = z.object({
  id: Id,
  handle: z.string(),
  name: z.optional(z.string()),
  description: z.optional(z.string()),
  host: z.string(),
  uri: z.string().url(),
  avatarUrl: z.optional(z.string().url()),
  bannerUrl: z.optional(z.string().url()),
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
  alt: z.optional(z.string()),
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
  user: z.optional(User),
  content: z.string(),
  emoji: z.optional(Emoji),
});

export const Post = z.object({
  id: Id,
  createdAt: z.string().datetime({ offset: true }),
  replyId: z.optional(Id),
  repliesId: z.array(Id),
  repostId: z.optional(Id),
  text: z.string(),
  title: z.optional(z.string()),
  user: z.optional(User),
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
  replyId: z.optional(Id),
  repostId: z.optional(Id),
  text: z.string(),
  title: z.optional(z.string()),
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
  emojiName: z.optional(z.string()),
  mediaType: z.string(),
  url: z.string().url(),
  alt: z.optional(z.string()),
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
  userName: z.optional(z.string()),
  userDescription: z.optional(z.string()),
  instanceDescription: z.optional(z.string()),
  avatarFileId: z.optional(Id),
  bannerFileId: z.optional(Id),
  maintainerName: z.optional(z.string()),
  maintainerEmail: z.optional(z.string().email()),
  themeColor: z.optional(z.string()),
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
