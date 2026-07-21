import { atom } from "jotai";

export interface ChatMessage {
  id: number;
  session_id: number;
  role: "user" | "assistant";
  content: string;
  created_at: string;
}

export interface ChatSession {
  id: number;
  title: string;
  app_instance_id: number | null;
  namespace: string;
  status: string;
  created_at: string;
  updated_at: string;
  messages: ChatMessage[];
}

export const currentSessionIdAtom = atom<number | null>(null);

export const messagesAtom = atom<ChatMessage[]>([]);

export const isGeneratingAtom = atom<boolean>(false);

export const prototypeUrlAtom = atom<string | null>(null);
