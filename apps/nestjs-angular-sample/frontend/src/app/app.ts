import { Component, OnInit, inject } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { AuthService } from './auth/auth.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet],
  template: `<router-outlet></router-outlet>`,
  styles: [`
    :host { display: block; }
  `],
})
export class App implements OnInit {
  private readonly auth = inject(AuthService);

  ngOnInit() {
    this.auth.restore();
  }
}
