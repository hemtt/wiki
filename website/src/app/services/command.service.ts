import { Injectable, signal, computed } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { Syntax } from 'src/bindings/Syntax';
import { Locality } from 'src/bindings/Locality';
import { Since } from 'src/bindings/Since';

export interface Command {
    name: string;
    id: string;
    description: string;
    groups: string[];
    status: 'Passed' | 'Failed' | 'Outdated' | 'Unknown';
    errors?: string[];
}

export interface FullCommand extends Command {
    examples: string[];
    alias?: string[];
    syntax: Syntax[];
    see_also: string[];
    since?: Since;
    argument_loc: Locality;
    effect_loc: Locality;
    problem_notes: string[];
}

export interface CommandsData {
    timestamp: number;
    version: string;
    commands: Command[];
    passed: number;
    failed: number;
    total: number;
}

export interface FilterOptions {
    groups: string[];
    statuses: string[];
    sortOptions: Array<{ value: string; label: string }>;
}

@Injectable({
    providedIn: 'root',
})
export class CommandService {
    // Signals for state management
    timestamp = signal(0);
    progress = signal(0);
    commands = signal<Command[]>([]);
    loading = signal(false);
    error = signal<string | null>(null);

    // Internal signal for search parameters
    private searchQuery = signal('');
    private searchStatus = signal('');
    private searchGroup = signal('');

    // Computed signal for filtered commands
    filteredCommands = computed(() => {
        const query = this.searchQuery();
        const status = this.searchStatus();
        const group = this.searchGroup();
        const commands = this.commands();

        return commands.filter((cmd) => {
            const matchesQuery =
                !query || cmd.name.toLowerCase().includes(query.toLowerCase());
            const matchesStatus = !status || cmd.status === status;
            const matchesGroup =
                !group || (cmd.groups && cmd.groups.includes(group));

            return matchesQuery && matchesStatus && matchesGroup;
        });
    });

    constructor(private http: HttpClient) {
        this.loadCommands();
    }

    private loadCommands(): void {
        this.loading.set(true);
        this.http.get<CommandsData>('assets/data/commands.json').subscribe({
            next: (data: CommandsData) => {
                this.commands.set(data.commands);
                console.log(`Loaded ${data.total} commands (Passed: ${data.passed}, Failed: ${data.failed})`);
                this.progress.set((data.passed / data.total) * 100);
                // seconds since epoch
                this.timestamp.set(data.timestamp * 1000);
                this.loading.set(false);
            },
            error: (err: unknown) => {
                console.error('Failed to load commands', err);
                this.error.set('Failed to load command data');
                this.loading.set(false);
            },
        });
    }

    getFilters(): Observable<FilterOptions> {
        return this.http.get<FilterOptions>('assets/data/filters.json');
    }

    updateSearch(query: string, status: string, group: string): void {
        this.searchQuery.set(query);
        this.searchStatus.set(status);
        this.searchGroup.set(group);
    }

    getFilteredCommands() {
        return this.filteredCommands;
    }

    loadCommandDetails(commandName: string): Observable<FullCommand> {
        const filename = commandName.replace(/ /g, '_');
        return this.http.get<FullCommand>(`assets/data/commands/${filename}.json`);
    }

    isCommand(name: string): boolean {
        const commands = this.commands();
        return commands.some((cmd) => cmd.name === name);
    }
}
