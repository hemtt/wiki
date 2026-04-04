import { Component, Input } from '@angular/core';
import { Value } from 'src/bindings/Value';
import { TypeComponent } from '../type/type.component';

@Component({
    selector: 'app-value',
    standalone: true,
    imports: [TypeComponent],
    templateUrl: './value.component.html',
})
export class ValueComponent {
    @Input() value!: Value;

    get getType(): string {
        if (!this.value) return '';
        if (typeof this.value === 'string') {
            return this.value;
        } else if (typeof this.value === 'object') {
            return Object.keys(this.value)[0];
        }
        return '';
    }
}

