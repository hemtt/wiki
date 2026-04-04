import { Component, OnInit, signal, computed } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { DatePipe } from '@angular/common';
import { CommandService, Command, FilterOptions } from '../../services/command.service';
import { CommandCardComponent } from '../command-card/command-card.component';

@Component({
    selector: 'app-command-viewer',
    standalone: true,
    imports: [FormsModule, DatePipe, CommandCardComponent],
    templateUrl: './command-viewer.component.html',
})
export class CommandViewerComponent implements OnInit {
    // State signals
    filters = signal<FilterOptions | null>(null);

    // Local filter signals
    searchQuery = signal('');
    selectedStatus = signal('');
    selectedGroup = signal('');
    sortBy = signal('name');

    // Computed signals for filter options
    filterStatuses = computed(() => this.filters()?.statuses ?? []);
    filterGroups = computed(() => this.filters()?.groups ?? []);
    filterSortOptions = computed(() => this.filters()?.sortOptions ?? []);

    // Access to service signals
    get commands() {
        return this.commandService.commands;
    }

    get timestamp() {
        return this.commandService.timestamp;
    }

    get progress() {
        return this.commandService.progress;
    }

    get loading() {
        return this.commandService.loading;
    }

    get error() {
        return this.commandService.error;
    }

    get statuses() {
        return this.filterStatuses;
    }

    get groups() {
        return this.filterGroups;
    }

    get sortOptions() {
        return this.filterSortOptions;
    }

    get filteredCommands() {
        const filtered = this.commandService.getFilteredCommands()();
        return this.sortCommands(filtered);
    }

    constructor(private commandService: CommandService) { }

    ngOnInit(): void {
        this.commandService.getFilters().subscribe((filters) => {
            this.filters.set(filters);
        });
    }

    private sortCommands(commands: Command[]): Command[] {
        const sorted = [...commands];
        const sort = this.sortBy();
        switch (sort) {
            case 'name':
                sorted.sort((a, b) => a.name.localeCompare(b.name));
                break;
            case 'name-desc':
                sorted.sort((a, b) => b.name.localeCompare(a.name));
                break;
            case 'status': {
                const statusOrder: Record<string, number> = {
                    Passed: 0,
                    Outdated: 1,
                    Failed: 2,
                    Unknown: 3,
                };
                sorted.sort(
                    (a, b) =>
                        (statusOrder[a.status] ?? 3) - (statusOrder[b.status] ?? 3),
                );
                break;
            }
            case 'group':
                sorted.sort(
                    (a, b) =>
                        (a.groups?.[0] ?? 'Z').localeCompare(b.groups?.[0] ?? 'Z'),
                );
                break;
        }
        return sorted;
    }

    onSearchChange(): void {
        this.commandService.updateSearch(
            this.searchQuery(),
            this.selectedStatus(),
            this.selectedGroup(),
        );
    }

    onStatusChange(): void {
        this.commandService.updateSearch(
            this.searchQuery(),
            this.selectedStatus(),
            this.selectedGroup(),
        );
    }

    onGroupChange(): void {
        this.commandService.updateSearch(
            this.searchQuery(),
            this.selectedStatus(),
            this.selectedGroup(),
        );
    }

    onSortChange(): void {
        // Sorting is handled in the getter
    }
}
