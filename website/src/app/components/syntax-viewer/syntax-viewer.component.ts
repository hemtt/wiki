import { Component, Input } from '@angular/core';
import { FullCommand } from 'src/app/services/command.service';
import { Param } from 'src/bindings/Param';
import { Syntax } from 'src/bindings/Syntax';
import { ReturnTypeComponent } from "../return-type/return-type.component";
import { TypeComponent } from '../type/type.component';

interface ParamInfo {
    name: string;
    type: string;
    desc?: string;
}

interface SyntaxInfo {
    signature: string;
    parameters: ParamInfo[];
}

@Component({
    selector: 'app-syntax-viewer',
    standalone: true,
    templateUrl: './syntax-viewer.component.html',
    imports: [ReturnTypeComponent, TypeComponent],
})
export class SyntaxViewerComponent {
    @Input() command!: FullCommand;
    @Input() syntax!: Syntax;

    get syntaxInfo(): SyntaxInfo {
        try {
            return this.parseSyntax(this.command, this.syntax);
        } catch (e) {
            console.error('Error parsing syntax:', e);
            return {
                signature: 'Unable to parse syntax',
                parameters: [],
            };
        }
    }

    private parseSyntax(command: FullCommand, syntax: Syntax): SyntaxInfo {
        if (!syntax || typeof syntax !== 'object') {
            return { signature: String(syntax), parameters: [] };
        }

        const signature = this.buildSignature(command, syntax);
        const parameters = this.extractParameters(syntax);

        console.log('Return type:', syntax.ret);

        return { signature, parameters };
    }

    private buildSignature(command: FullCommand, syntax: Syntax): string {
        if (typeof syntax.call === 'object' && 'Binary' in syntax.call) {
            return `${this.buildSignatureArg(syntax.left!)} <span class="text-orange-700">${command.name}</span> ${this.buildSignatureArg(syntax.right!)}`;
        } else if (syntax.call === 'Nular') {
            return `<span class="text-orange-700">${command.name}</span>`;
        } else if (typeof syntax.call === 'object' && 'Unary' in syntax.call) {
            return `<span class="text-orange-700">${command.name}</span> ${this.buildSignatureArg(syntax.right!)}`;
        }
        return `<span class="text-orange-700">${command.name}</span>`;
    }

    private buildSignatureArg(arg: Param): string {
        if ('Item' in arg && arg.Item) {
            return arg.Item.name;
        }
        if ('Array' in arg && arg.Array) {
            let items = arg.Array.map((item: Param) => this.formatType(item)).join(', ');
            return `[${items}]`;
        }
        return `{ Unknown }`;
    }

    private extractParameters(syntax: Syntax): ParamInfo[] {
        const params: ParamInfo[] = [];

        // Check common parameter locations
        if (syntax.left) {
            const leftParam = this.extractParamFromObject(
                syntax.left,
                'left operand',
            );
            if (leftParam) params.push(...leftParam);
        }

        if (syntax.right) {
            const rightParam = this.extractParamFromObject(
                syntax.right,
                'right operand',
            );
            if (rightParam) params.push(...rightParam);
        }

        return params;
    }

    private extractParamFromObject(
        obj: Param,
        defaultName: string,
    ): ParamInfo[] {
        if (!obj || typeof obj !== 'object') {
            console.error('Invalid parameter object:', obj);
            return [];
        }

        // Look for Item with name and type
        if ('Item' in obj) {
            const item = obj.Item;
            const name = item.name || defaultName;
            const type = this.formatType(item.type) || 'Unknown';
            return [{ name, type, desc: item.desc || undefined }];
        }
        
        // Look for Array
        if ('Array' in obj) {
            const items: ParamInfo[] = obj.Array.map((item: Param, index: number) => {
                if ('Item' in item) {
                    const itemData = item.Item;
                    return {
                        name: itemData.name || `${defaultName} ${index + 1}`,
                        type: this.formatType(itemData.type) || 'Unknown',
                        desc: itemData.desc || undefined,
                    };
                }
                return { name: `${defaultName} ${index + 1}`, type: 'Unknown' };
            });
            return items;
        }
        
        // Look for Infinite
        if ('Infinite' in obj) {
            const items: ParamInfo[] = obj.Infinite.map((item: Param, index: number) => {
                if ('Item' in item) {
                    const itemData = item.Item;
                    return {
                        name: itemData.name || `${defaultName} ${index + 1}`,
                        type: this.formatType(itemData.type) || 'Unknown',
                        desc: itemData.desc || undefined,
                    };
                }
                return { name: `${defaultName} ${index + 1}`, type: 'Unknown' };
            });
            return items;
        }

        console.warn('Unable to extract parameter info from object:', obj);
        return [];
    }

    private formatType(value: any): string {
        if (!value) return '';

        if (typeof value === 'string') {
            return value;
        }

        if (typeof value === 'object') {
            if (value.Binary && Array.isArray(value.Binary)) {
                return value.Binary[0];
            }
            if (value.Item && value.Item.name) {
                return value.Item.name;
            }
            if (value.ArrayUnsized) {
                return `Array<${this.formatType(value.ArrayUnsized.value)}>`;
            }
            if (value.ArraySized) {
                let items: string[] = [];
                value.ArraySized.forEach((item: any) => {
                    
                });
                return `Array<${items.join(', ')}>`;
            }
            if (value.type) {
                return value.type;
            }
        }

        return '';
    }

    private inferType(obj: any): string {
        if (!obj || typeof obj !== 'object') {
            return typeof obj;
        }

        if (obj.Binary) return 'Value';
        if (obj.Item) return 'Object';
        if (obj.type) return obj.type;

        return 'Unknown';
    }
}

