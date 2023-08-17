export interface IdResponse {
  id: string;
}

export interface NameResponse {
  name: string;
}

export interface User {
  id: string;
  handle: string;
  name?: string;
  description?: string;
  host: string;
  uri: string;
  avatarUrl?: string;
  bannerUrl?: string;
  manuallyApprovesFollowers: boolean;
  isBot: boolean;
}

export type Visibility = "public" | "home" | "followers" | "directMessage";

export interface Mention {
  userUri: string;
  name: string;
}

export interface File {
  mediaType: string;
  url: string;
  alt?: string;
}

export interface Emoji {
  name: string;
  mediaType: string;
  imageUrl: string;
}

export interface CreateContentReaction {
  content: string;
}

export interface CreateEmojiReaction {
  emojiName: string;
}

export type CreateReaction = CreateContentReaction | CreateEmojiReaction;

export interface Reaction {
  id: string;
  user?: User;
  content: string;
  emoji?: Emoji;
}

export interface Post {
  id: string;
  createdAt: string;
  replyId?: string;
  repliesId: string[];
  repostId?: string;
  text: string;
  title?: string;
  user?: User;
  visibility: Visibility;
  isSensitive: boolean;
  uri: string;
  files: File[];
  reactions: Reaction[];
  mentions: Mention[];
  emojis: Emoji[];
  hashtags: string[];
}

export interface CreatePost {
  replyId?: string;
  repostId?: string;
  text: string;
  title?: string;
  visibility: Visibility;
  isSensitive: boolean;
  files: string[];
  mentions: Mention[];
  emojis: string[];
  hashtags: string[];
}

export interface LocalFile {
  id: string;
  posted: boolean;
  emojiName?: string;
  mediaType: string;
  url: string;
  alt?: string;
}

export interface LocalEmoji {
  name: string;
  createdAt: string;
  mediaType: string;
  imageUrl: string;
}

export interface CreateEmoji {
  fileId: string;
  name: string;
}

export type Follow = User & { accepted: boolean };

export interface CreateFollow {
  toId: string;
}

export interface Setting {
  userName?: string;
  userDescription?: string;
  instanceDescription?: string;
  avatarFileId?: string;
  bannerFileId?: string;
  maintainerName?: string;
  maintainerEmail?: string;
  themeColor?: string;
}

export type Object = User | Post;

export interface Report {
  from: User;
  content: string;
}

export interface CreateReport {
  userId: string;
  content: string;
}
