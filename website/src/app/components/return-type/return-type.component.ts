import { Component, Input } from '@angular/core';
import { Return } from 'src/bindings/Return';
import { ValueComponent } from "../value/value.component";

@Component({
    selector: 'app-return-type',
    standalone: true,
    templateUrl: './return-type.component.html',
    imports: [ValueComponent],
})
export class ReturnTypeComponent {
    @Input() return!: Return;

    get getType(): string {
        if (!this.return) return '';
        if (typeof this.return === 'string') {
            return this.return;
        } else if (typeof this.return === 'object' && 'type' in this.return) {
            return Object.keys(this.return.type)[0];
        }
        return '';
    }
}

