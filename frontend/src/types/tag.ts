export interface TagSummaryDto {
  id: string;
  name: string;
  color: string;
}

export interface TagDetailDto {
  id: string;
  name: string;
  color: string;
}

export interface CreateTagDto {
  name: string;
  color: string;
}

export interface UpdateTagDto {
  name: string;
  color: string;
}
