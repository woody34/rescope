import { Component, OnInit, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { HttpClient } from '@angular/common/http';
import { environment } from '../../environments/environment';

interface Todo {
  id: string;
  text: string;
  createdAt: string;
  createdBy: string;
}

@Component({
  selector: 'app-todo-table',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="todo-section">
      <h3>📋 Todos</h3>

      <div class="add-todo">
        <input
          [value]="newTodoText()"
          (input)="newTodoText.set($any($event.target).value)"
          placeholder="Add a todo..."
          (keydown.enter)="addTodo()"
          id="todo-input"
        />
        <button (click)="addTodo()" id="add-todo-btn">Add</button>
      </div>

      @if (loading()) {
        <p class="loading">Loading todos...</p>
      } @else if (todos().length === 0) {
        <p class="empty">No todos yet. Add one above.</p>
      } @else {
        <table id="todo-table">
          <thead>
            <tr>
              <th>Task</th>
              <th>Created</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            @for (todo of todos(); track todo.id) {
              <tr [attr.data-todo-id]="todo.id">
                <td>{{ todo.text }}</td>
                <td>{{ todo.createdAt | date:'short' }}</td>
                <td>
                  <button
                    class="delete-btn"
                    (click)="deleteTodo(todo.id)"
                    [attr.data-testid]="'delete-' + todo.id"
                  >Delete</button>
                </td>
              </tr>
            }
          </tbody>
        </table>
      }

      @if (error()) {
        <p class="error" id="todo-error">{{ error() }}</p>
      }
    </div>
  `,
  styles: [`
    .todo-section { margin-top: 1rem; }
    .add-todo { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
    .add-todo input { flex: 1; padding: 0.5rem; border-radius: 4px; border: 1px solid #ccc; }
    button { padding: 0.4rem 0.8rem; border-radius: 4px; cursor: pointer; border: 1px solid #ccc; }
    table { width: 100%; border-collapse: collapse; }
    th, td { padding: 0.5rem; border: 1px solid #ddd; text-align: left; }
    th { background: #f0f0f0; }
    .delete-btn { background: #fee; border-color: #f99; }
    .error { color: red; }
    .loading, .empty { color: #888; font-style: italic; }
  `],
})
export class TodoTableComponent implements OnInit {
  private readonly http = inject(HttpClient);
  todos = signal<Todo[]>([]);
  newTodoText = signal('');
  loading = signal(false);
  error = signal('');

  ngOnInit() {
    this.loadTodos();
  }

  loadTodos() {
    this.loading.set(true);
    this.error.set('');
    this.http.get<Todo[]>(`${environment.apiBaseUrl}/api/todos`).subscribe({
      next: (todos) => { this.todos.set(todos); this.loading.set(false); },
      error: (err) => {
        this.error.set(
          err.status === 401 ? 'Not authenticated — log in first.' :
          err.status === 403 ? 'Access denied.' :
          `Error: ${err.message}`
        );
        this.loading.set(false);
      },
    });
  }

  addTodo() {
    if (!this.newTodoText().trim()) return;
    this.http.post<Todo>(`${environment.apiBaseUrl}/api/todos`, { text: this.newTodoText() }).subscribe({
      next: (todo) => { this.todos.update(ts => [...ts, todo]); this.newTodoText.set(''); },
      error: (err) => { this.error.set(`Add failed: ${err.message}`); },
    });
  }

  deleteTodo(id: string) {
    this.http.delete(`${environment.apiBaseUrl}/api/todos/${id}`).subscribe({
      next: () => { this.todos.update(ts => ts.filter(t => t.id !== id)); },
      error: (err) => {
        this.error.set(err.status === 403 ? 'Delete requires admin role.' : `Delete failed: ${err.message}`);
      },
    });
  }
}
