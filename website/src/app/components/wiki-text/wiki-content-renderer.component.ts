import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { WikiLinkComponent } from './wiki-link.component';
import { WikiTemplateComponent } from "./wiki-template.component";

@Component({
    selector: 'app-wiki-content-renderer',
    standalone: true,
    imports: [CommonModule, WikiLinkComponent, WikiTemplateComponent],
    template: `
    @for (node of nodes; track node) {
        @switch (node.type) {
            @case ('text') {
                {{ node.data }}
            }
            @case ('html') {
                <div [innerHTML]="'<' + node.name + '>'"></div>
            }
            @case ('link') {
                <app-wiki-link [node]="node"></app-wiki-link>
            }
            @case ('list-group') {
                <ul class="list-disc list-inside my-2">
                    @for (item of node.childNodes; track item) {
                        <li>
                            <app-wiki-content-renderer [nodes]="item.childNodes"></app-wiki-content-renderer>
                        </li>
                    }
                </ul>
            }
            @case ('template') {
                <app-wiki-template [template]="node"></app-wiki-template>
            }
            @case ('quote') {
                <span class="font-bold">
                    <app-wiki-content-renderer [nodes]="node.childNodes"></app-wiki-content-renderer>
                </span>
            }
            @case ('file') {}
            @default {
                <b>{{ node.type }}</b>
            }
        }
    }
    `,
})
export class WikiContentRendererComponent {
    @Input() nodes: any[] = [];
}
