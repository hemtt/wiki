import { Component, Input } from '@angular/core';
import { Router } from '@angular/router';
import { Command } from '../../services/command.service';

@Component({
  selector: 'app-command-card',
  standalone: true,
  templateUrl: './command-card.component.html',
  styleUrls: ['./command-card.component.css'],
})
export class CommandCardComponent {
  @Input() command!: Command;

  constructor(private router: Router) {}

  getStatusClasses(): string {
    const baseClasses = 'px-3 py-1 rounded text-xs font-semibold';
    const statusClasses: Record<string, string> = {
      Passed: 'bg-green-100 text-green-800',
      Failed: 'bg-red-100 text-red-800',
      Outdated: 'bg-yellow-100 text-yellow-800',
      Unknown: 'bg-gray-100 text-gray-800',
    };
    return `${baseClasses} ${statusClasses[this.command.status] || statusClasses['Unknown']}`;
  }

  viewDetails(): void {
    this.router.navigate(['/command', this.command.name]);
  }
}
