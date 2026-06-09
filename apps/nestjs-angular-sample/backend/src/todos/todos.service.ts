import { Injectable } from '@nestjs/common';

export interface Todo {
  id: string;
  text: string;
  createdAt: string;
  createdBy: string;
}

@Injectable()
export class TodosService {
  /** Key = dct (tenant) if present, otherwise sub (user) */
  private readonly store = new Map<string, Todo[]>();

  private bucketFor(sub: string, dct?: string): string {
    return dct ?? sub;
  }

  findAll(sub: string, dct?: string): Todo[] {
    const key = this.bucketFor(sub, dct);
    return this.store.get(key) ?? [];
  }

  create(text: string, sub: string, dct?: string): Todo {
    const key = this.bucketFor(sub, dct);
    const todo: Todo = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
      text,
      createdAt: new Date().toISOString(),
      createdBy: sub,
    };
    const current = this.store.get(key) ?? [];
    this.store.set(key, [...current, todo]);
    return todo;
  }

  delete(id: string, sub: string, dct?: string): boolean {
    const key = this.bucketFor(sub, dct);
    const todos = this.store.get(key) ?? [];
    const filtered = todos.filter((t) => t.id !== id);
    if (filtered.length === todos.length) return false;
    this.store.set(key, filtered);
    return true;
  }
}
