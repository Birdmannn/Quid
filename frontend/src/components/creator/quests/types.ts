export type QuestTabKey = "active" | "drafted" | "completed";

export type QuestTagVariant = "active" | "draft" | "completed";

export interface QuestRowData {
  id: string;
  title: string;
  tagLabel: string;
  tagVariant: QuestTagVariant;
  category: string;
  pool: number;
  perWinner: number;
  responses: number;
  meta: string;
  tab: QuestTabKey;
}
