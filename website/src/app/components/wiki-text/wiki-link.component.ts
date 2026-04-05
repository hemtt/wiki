import { Component, Input, OnChanges, signal, SimpleChanges } from "@angular/core";
import { RouterLink } from "@angular/router";
import { CommandService } from "src/app/services/command.service";

@Component({
    selector: 'app-wiki-link',
    standalone: true,
    imports: [RouterLink],
    template: `
    @switch (destination()) {
        @case ('bi') {
            <a class="hover:underline text-orange-500" [href]="'https://community.bistudio.com/wiki/' + href()" target="_blank" rel="noopener noreferrer">{{ display() }} </a>
        }
        @case ('command') {
            <a class="hover:underline text-blue-500" [routerLink]="['/command', href()]">{{ display() }}</a>
        }
    }
    `,
})
export class WikiLinkComponent implements OnChanges {
    @Input() node!: any;

    protected href = signal('');
    protected display = signal('');
    protected destination = signal('bi');

    constructor(
        private commandService: CommandService,
    ) {}

    ngOnChanges(changes: SimpleChanges) {
        if (changes['node'] && this.node) {
            this.updateLink();
        }
    }

    private updateLink() {
        if (this.node.type !== 'link') {
            console.warn('Node is not a link:', this.node);
            return;
        }

        let href = '';
        let display = '';

        this.node.childNodes.forEach((child: any) => {
            switch (child.type) {
                case 'link-target':
                    href = child.childNodes[0]?.data || '';
                    break;
                case 'link-text':
                    display = child.childNodes[0]?.data || '';
                    break;
            }
        });

        if (display === '' && href) {
            display = href;
        }

        this.href.set(href);
        this.display.set(display);

        if (this.commandService.isCommand(href)) {
            this.destination.set('command');
        }
    }
}
