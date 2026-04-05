import { Component, forwardRef, Input } from '@angular/core';
import { WikiContentRendererComponent } from "./wiki-content-renderer.component";
import { WikiTemplateFeatureComponent } from "./wiki-template-feature.component";

@Component({
    selector: 'app-wiki-template',
    standalone: true,
    imports: [forwardRef(() => WikiContentRendererComponent), WikiTemplateFeatureComponent],
    template: `
    @switch (name) {
        @case ('Feature') {
            <app-wiki-template-feature [template]="template"></app-wiki-template-feature>
        }
        @case ('hl') {
            <pre class="inline-block"><app-wiki-content-renderer [nodes]="template.childNodes[1].childNodes[1].childNodes"></app-wiki-content-renderer></pre>
        }
        @default {
            <div>
                <h3>{{ name }}</h3>
                <app-wiki-content-renderer [nodes]="template.childNodes.slice(1)"></app-wiki-content-renderer>
            </div>
        }
    }
    `
})
export class WikiTemplateComponent {
    @Input() template!: any;

    get name(): string {
        console.log('Template:', this.template.childNodes[0].childNodes[0].data);
        return this.template.childNodes[0].childNodes[0].data;
    }
}

