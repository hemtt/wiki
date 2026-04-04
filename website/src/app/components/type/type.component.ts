import { Component, Input } from '@angular/core';
import { Value } from 'src/bindings/Value';

@Component({
    selector: 'app-type',
    standalone: true,
    template: `<a target="_blank" [href]="'https://community.bistudio.com/wiki/' + getLink" class="text-violet-600 italic">{{ type }}</a>`
})
export class TypeComponent {
    @Input() type!: string;

    get getLink(): string {
        return this.type;
    }
}

