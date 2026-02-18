export type ResultType = "App" | "Folder" | "Image";

export interface AppResult {
  name: string;
  exec: string;
  icon: string | null;
  description: string | null;
  result_type: ResultType;
}
