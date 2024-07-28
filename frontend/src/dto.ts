import * as z from "zod";

export const Id = z.string().ulid();

export const ErrorResponse = z.object({
  id: Id,
  error: z.string(),
});

export async function throwError(resp: Response): Promise<never> {
  const body = await resp.text();
  let error;
  try {
    const parsed = ErrorResponse.parse(JSON.parse(body));
    error = new Error(
      `failed to request. status code: ${resp.status}, error id: ${parsed.id}, message: ${parsed.error}`,
    );
  } catch {
    error = new Error(
      `failed to request. status code: ${resp.status}, message: ${body}`,
    );
  }
  throw error;
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
  name: z.string().nullish(),
  description: z.string().nullish(),
  host: z.string(),
  uri: z.string().url(),
  avatarUrl: z.string().url().nullish(),
  bannerUrl: z.string().url().nullish(),
  manuallyApprovesFollowers: z.boolean(),
  isBot: z.boolean(),
});

export const Visibility = z.enum([
  "public",
  "home",
  "followers",
  "directMessage",
]);

export const ObjectStoreType = z.enum(["s3", "localFileSystem"]);

export const Mention = z.object({
  userUri: z.string().url(),
  name: z.string(),
});

export const File = z.object({
  mediaType: z.string(),
  url: z.string().url(),
  alt: z.string().nullish(),
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

export const CreateReaction = z.union([
  CreateContentReaction,
  CreateEmojiReaction,
]);

export const Reaction = z.object({
  id: Id,
  user: User.nullish(),
  content: z.string(),
  emoji: Emoji.nullish(),
});

export const Post = z.object({
  id: Id,
  createdAt: z.string().datetime({ offset: true }),
  replyId: Id.nullish(),
  repliesId: z.array(Id),
  repostId: Id.nullish(),
  text: z.string(),
  title: z.string().nullish(),
  user: User.nullish(),
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
  replyId: Id.nullish(),
  repostId: Id.nullish(),
  text: z.string(),
  title: z.string().nullish(),
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
  emojiName: z.string().nullish(),
  mediaType: z.string(),
  url: z.string().url(),
  alt: z.string().nullish(),
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
  userHandle: z.string(),
  userName: z.string().nullish(),
  userDescription: z.string().nullish(),
  instanceName: z.string(),
  instanceDescription: z.string().nullish(),
  avatarFileId: Id.nullish(),
  bannerFileId: Id.nullish(),
  maintainerName: z.string().nullish(),
  maintainerEmail: z.string().nullish(),
  themeColor: z.string().nullish(),
  objectStoreType: ObjectStoreType.nullish(),
  objectStoreS3Bucket: z.string().nullish(),
  objectStoreS3PublicUrlBase: z.string().url().nullish(),
  objectStoreLocalFileSystemBasePath: z.string().nullish(),
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

export const Update = z.union([
  z.object({
    type: z.literal("createPost"),
    postId: Id,
  }),
  z.object({
    type: z.literal("deletePost"),
    postId: Id,
  }),
  z.object({
    type: z.literal("createReaction"),
    postId: Id,
  }),
  z.object({
    type: z.literal("deleteReaction"),
    postId: Id,
  }),
  z.object({
    type: z.literal("updateUser"),
    userId: Id,
  }),
  z.object({
    type: z.literal("deleteUser"),
    userId: Id,
  }),
]);

export const Notification = z.intersection(
  z.object({ id: Id }),
  z.union([
    z.object({
      type: z.literal("acceptFollow"),
      userId: Id,
    }),
    z.object({
      type: z.literal("rejectFollow"),
      userId: Id,
    }),
    z.object({
      type: z.literal("createFollower"),
      userId: Id,
    }),
    z.object({
      type: z.literal("deleteFollower"),
      userId: Id,
    }),
    z.object({
      type: z.literal("createReport"),
      reportId: Id,
    }),
    z.object({
      type: z.literal("mentioned"),
      postId: Id,
    }),
    z.object({
      type: z.literal("reposted"),
      postId: Id,
    }),
    z.object({
      type: z.literal("quoted"),
      postId: Id,
    }),
    z.object({
      type: z.literal("reacted"),
      postId: Id,
      reactionId: Id,
    }),
  ]),
);

export const Event = z.union([
  z.intersection(
    z.object({
      eventType: z.literal("update"),
    }),
    Update,
  ),
  z.intersection(
    z.object({
      eventType: z.literal("notification"),
    }),
    Notification,
  ),
]);
