import { Component, OnInit, signal, untracked, effect, computed } from '@angular/core';
import { Router, ActivatedRoute, RouterLink } from '@angular/router';
import { CommandService, Command, FullCommand } from '../../services/command.service';
import { SyntaxViewerComponent } from '../syntax-viewer/syntax-viewer.component';
import { GameVersionComponent } from '../game-version/game-version.component';
import { mapSince } from 'src/app/mapSince';
import { WikiTextComponent } from "../wiki-text/wiki-text.component";

@Component({
    selector: 'app-command-detail',
    standalone: true,
    imports: [RouterLink, SyntaxViewerComponent, GameVersionComponent, WikiTextComponent],
    templateUrl: './command-detail.component.html',
})
export class CommandDetailComponent implements OnInit {
    basicCommand = signal<Command | null>(null);
    fullCommand = signal<FullCommand | null>(null);
    loading = signal(true);
    error = signal(false);

    // Computed signals for command properties that might be undefined
    commandErrors = computed(() => this.fullCommand()?.errors ?? this.basicCommand()?.errors ?? []);
    commandGroups = computed(() => this.fullCommand()?.groups ?? []);
    commandSyntax = computed(() => this.fullCommand()?.syntax ?? []);
    commandSince = computed(() => {
        if (this.fullCommand()?.since) {
            const since = this.fullCommand()!.since!;
            return mapSince(since);
        } else {
            return [];
        }
    });
    commandExamples = computed(() => {
        return (this.fullCommand()?.examples ?? []).map(example => {
            example = example.replaceAll("<sqf", "<div class=\"bg-blue-100 font-mono whitespace-pre-wrap p-2 rounded\"").replaceAll("</sqf>", "</div>");
            // Remove leading newlines
            example = example.replaceAll(/>\s*\n/g, '>');
            return example;
        });
    });
    commandProblemNotes = computed(() => this.fullCommand()?.problem_notes ?? []);
    commandSeeAlso = computed(() => this.fullCommand()?.see_also ?? []);
    commandArgumentLoc = computed(() => this.fullCommand()?.argument_loc ?? '');
    commandEffectLoc = computed(() => this.fullCommand()?.effect_loc ?? '');

    get command(): Command | null {
        return this.fullCommand() || this.basicCommand();
    }

    constructor(
        private commandService: CommandService,
        private route: ActivatedRoute,
        private router: Router,
    ) {
        // Set up effect to react to route param changes
        effect(() => {
            const params = untracked(() => this.route.snapshot.params);
            const commandName = params['name'];
            if (commandName) {
                this.loadCommand(commandName);
            }
        });
    }

    ngOnInit(): void {
        // Subscribe to route params dynamically
        this.route.params.subscribe((params) => {
            const commandName = params['name'];
            if (commandName) {
                this.loadCommand(commandName);
            }
        });
    }

    private loadCommand(name: string): void {
        this.loading.set(true);
        this.error.set(false);
        this.basicCommand.set(null);
        this.fullCommand.set(null);

        const commands = this.commandService.commands();
        const found = commands.find(
            (c) => c.id === name,
        );

        if (!found) {
            this.error.set(true);
            this.loading.set(false);
            return;
        }

        this.basicCommand.set(found);

        // If command is passed, load full details
        if (found.status === 'Passed') {
            this.commandService.loadCommandDetails(found.id).subscribe({
                next: (fullCmd) => {
                    this.fullCommand.set(fullCmd);
                    this.loading.set(false);
                },
                error: (err) => {
                    console.error('Failed to load full command details', err);
                    this.loading.set(false);
                },
            });
        } else {
            this.loading.set(false);
        }
    }

    goBack(): void {
        this.router.navigate(['/']);
    }

    getStatusClasses(): string {
        const cmd = this.command;
        if (!cmd) return '';
        const baseClasses = 'px-4 py-2 rounded text-sm font-semibold';
        const statusClasses: Record<string, string> = {
            Passed: 'bg-green-100 text-green-800',
            Failed: 'bg-red-100 text-red-800',
            Outdated: 'bg-yellow-100 text-yellow-800',
            Unknown: 'bg-gray-100 text-gray-800',
        };
        return `${baseClasses} ${statusClasses[cmd.status] || statusClasses['Unknown']}`;
    }
}
