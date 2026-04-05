import { Component, computed, Input, forwardRef } from '@angular/core';
import { WikiContentRendererComponent } from "./wiki-content-renderer.component";

@Component({
    selector: 'app-wiki-template-feature',
    standalone: true,
    imports: [forwardRef(() => WikiContentRendererComponent)],
    template: `
        <div class="my-6" [class]="containerClasses()">
            <app-wiki-content-renderer [nodes]="content()"></app-wiki-content-renderer>
        </div>
    `
})
export class WikiTemplateFeatureComponent {
    @Input() template: any;

    type = computed(() => {
        if (!this.template) return '';
        return this.template.childNodes[1].childNodes[1].childNodes[0].data.trim().toLowerCase();
    });
    content = computed(() => {
        if (!this.template) return [];
        return this.template.childNodes[2].childNodes[1].childNodes;
    });
    containerClasses = computed(() => {
        const type = this.type();
        console.log('Feature type:', type);
        switch (type) {
            case 'important':
                return 'border-l-4 border-yellow-500 bg-yellow-50 p-4';
            default:
                return '';
        }
    });
}

